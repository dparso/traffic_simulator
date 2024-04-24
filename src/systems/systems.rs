use bevy::{
    math::bounding::{Aabb2d, Bounded2d, BoundingVolume, IntersectsVolume, RayCast2d},
    prelude::*,
    render::primitives::Aabb,
    utils::hashbrown::HashMap,
    window::PrimaryWindow,
};

use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use crate::util::*;

pub fn debug_mouse_system(
    cursor_coords: ResMut<CursorWorldCoords>,
    mut query: Query<(&mut Text, &mut Style), With<MouseText>>,
) {
    let text_position = screen_space_to_world_coords(&cursor_coords.0);

    let (mut text, mut style) = query.single_mut();

    text.sections[0].value = get_mouse_text(&cursor_coords.0, &text_position);

    // so mouse doesn't cover text
    let buffer = Vec2::new(10., 10.);

    // TODO: would like to get text width to max position against end of screen
    style.top = Val::Px(text_position.y + buffer.y);
    style.left = Val::Px(text_position.x + buffer.x);
}

pub fn cursor_system(
    mut cursor_coords: ResMut<CursorWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_coords.0 = world_position;
    }
}

pub fn apply_friction(mut query: Query<&mut Velocity, With<Friction>>) {
    for mut velocity in &mut query {
        velocity.x *= FRICTION_DECAY;
        velocity.y *= FRICTION_DECAY;
    }
}

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

pub fn wrap_position(mut query: Query<&mut Transform, With<Velocity>>) {
    for mut transform in &mut query {
        if transform.translation.y > TOP_WALL - (transform.scale.y / 2.0) {
            transform.translation.y = BOTTOM_WALL + (transform.scale.y / 2.0);
        }
    }
}

pub fn update_scoreboard(
    scoreboard: Res<Scoreboard>,
    mut query: Query<&mut Text, With<ScoreboardUi>>,
) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

pub fn draw_car_sight_lines(query: Query<&Transform, With<Car>>, mut gizmos: Gizmos) {
    for transform in &query {
        let line_start = get_car_front_middle(transform);

        gizmos.ray_2d(line_start, Vec2::Y * CAR_SIGHT_DISTANCE, Color::GREEN);
    }
}

pub fn agent_drive_system(mut query: Query<(&mut DriverAgent, &mut Velocity, &Transform)>) {
    for (mut agent, mut velocity, transform) in &mut query {
        agent_accelerate_or_brake(&mut agent, &mut velocity);

        // match agent.driver_state {
        //     DriverState::Normal => agent_normal_behavior(&mut agent, &mut velocity, &transform),
        //     DriverState::ChangingLanes => {
        //         agent_changing_lanes_behavior(&mut agent, &mut velocity, &transform)
        //     }
        // }
    }
}

pub fn agent_check_lane_change_system(
    mut commands: Commands,
    query: Query<(Entity, &mut DriverAgent, &Velocity, &Transform, &LaneEntity)>,
) {
    let mut lane_to_transform_map: HashMap<i32, &Transform> = Default::default();

    for (_, _, _, transform, lane) in &query {
        lane_to_transform_map.insert(lane.0, &transform);
    }

    for (entity, agent, velocity, transform, lane) in &query {
        match get_lane_change_direction(&agent, &velocity, lane.0) {
            LaneChangeDirection::Left => {
                attempt_lane_change(
                    entity,
                    LaneChangeDirection::Left,
                    &transform,
                    lane.0,
                    &lane_to_transform_map,
                    &mut commands,
                );
            }
            LaneChangeDirection::Right => {
                attempt_lane_change(
                    entity,
                    LaneChangeDirection::Right,
                    &transform,
                    lane.0,
                    &lane_to_transform_map,
                    &mut commands,
                );
            }
            LaneChangeDirection::None => {}
        }
    }
}

pub fn agent_active_lane_change_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut DriverAgent,
        &mut Velocity,
        &Transform,
        &ActiveLaneChange,
    )>,
) {
    for (entity, mut agent, mut velocity, transform, active_lane_change) in &mut query {
        // TODO: negative velocity
        // let current_lane_idx = lane_idx_from_screen_pos(transform.translation);
        // println!("moving to target lane {}", active_lane_change.lane_target);
        let target_lane_center = lane_idx_to_center(active_lane_change.lane_target);
        let distance_from_lane_center = target_lane_center.x - transform.translation.x;

        if distance_from_lane_center > 0.5 {
            let vec = Vec3::new(distance_from_lane_center, 0., 0.);
            let normalized = vec.normalize();
            velocity.x = normalized.x * CAR_GAS_POWER;
            // println!(
            //     "moving towards lane center={} at speed={} distance={}",
            //     target_lane_center, velocity.x, distance_from_lane_center
            // );
        } else {
            velocity.x = 0.;
            // agent.0 = DriverState::Normal;
            commands.entity(entity).remove::<ActiveLaneChange>();
        }
    }
}

fn get_lane_change_direction(
    agent: &DriverAgent,
    velocity: &Velocity,
    lane_idx: i32,
) -> LaneChangeDirection {
    // return the direction the agent wants to change langes; does not check for feasability
    // drivers want to change lanes in two scenarios:
    //  1) there's a car in front of them, and they're impatient
    //  2) they're law-abiding and want to move to the right lane when not passing

    let min_speed_threshold = driver_patience_min_speed_pct(&agent.patience);

    if has_obstacle_in_range(&agent) && velocity.y < min_speed_threshold {
        // println!(
        //     "I want to pass you! velocity={} threshold={}",
        //     velocity.y, min_speed_threshold
        // );

        return LaneChangeDirection::Left;
    } else {
        match agent.lawfulness {
            DriverLawfulness::Chaotic => {
                return LaneChangeDirection::None;
            }
            DriverLawfulness::Orderly => {
                if lane_idx < NUM_LANES - 1 {
                    // println!("I want to return to the right lane!");
                    return LaneChangeDirection::Right;
                }

                // println!("Orderly car is already at {} max={}", lane_idx, NUM_LANES);

                return LaneChangeDirection::None;
            }
        }
    }
}

fn attempt_lane_change(
    entity: Entity,
    direction: LaneChangeDirection,
    transform: &Transform,
    current_lane: i32,
    lane_to_transform_map: &HashMap<i32, &Transform>,
    commands: &mut Commands,
) {
    let next_lane = match direction {
        LaneChangeDirection::Left => current_lane - 1,
        LaneChangeDirection::Right => current_lane + 1,
        LaneChangeDirection::None => current_lane,
    };

    if is_lane_open(next_lane, &transform, &lane_to_transform_map) {
        // println!("Lane {} is open, I want to move there!", next_lane);
        // adding the ActiveLangeChange component means this entity will be
        // picked up by the LaneChangeSystem and its velocity modified
        commands.entity(entity).insert(ActiveLaneChange {
            lane_change_direction: direction,
            lane_target: next_lane,
        });
    } else {
        // println!("Lane {} is NOT open, but I want to move there!", next_lane);
    }
}

fn agent_accelerate_or_brake(agent: &mut DriverAgent, mut velocity: &mut Velocity) {
    if has_obstacle_in_range(&agent) {
        brake_for_front(&agent, &mut velocity);
    } else {
        let top_speed = SPEED_LIMIT * driver_temperament_top_speed_pct(&agent.temperament);
        if velocity.y < top_speed {
            velocity.y += CAR_GAS_POWER;
        }
    }
}

fn has_obstacle_in_range(agent: &DriverAgent) -> bool {
    agent.collision_information.front_distance > -1.
}

fn is_lane_open(
    target_lane: i32,
    car_pos: &Transform,
    lane_to_transform_map: &HashMap<i32, &Transform>,
) -> bool {
    // draw a box extending two lane widths to either side (will still only check intersection against cars in directly adjacent lane)
    //    _____
    //    |   |
    //    |Car|
    //    |   |
    //    ‾‾‾‾‾
    //    ==>
    //    _________________
    //    | ... |   | ... |
    //    | ... |Car| ... |
    //    | ... |   | ... |
    //    ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾

    // create adjusted size box to compare against cars in target lane
    let bottom_left = Vec2::new(
        car_pos.translation.x - LANE_WIDTH_DOUBLE,
        car_pos.translation.y - CAR_SIZE_HALF.y,
    );

    let top_right = Vec2::new(
        car_pos.translation.x + LANE_WIDTH_DOUBLE,
        car_pos.translation.y + CAR_SIZE_HALF.y,
    );

    let bounding_box = Aabb2d {
        min: bottom_left,
        max: top_right,
    };

    for (lane_idx, &target_transform) in lane_to_transform_map {
        if *lane_idx != target_lane {
            continue;
        }

        if bounding_box.intersects(&Aabb2d::new(
            target_transform.translation.truncate(),
            CAR_SIZE_HALF.truncate(),
        )) {
            return false;
        }
    }

    true
}

fn brake_for_front(agent: &DriverAgent, velocity: &mut Velocity) {
    let distance = agent.collision_information.front_distance;
    let brake_distance_threshold =
        CAR_SIGHT_DISTANCE * driver_temperament_brake_threshold(&agent.temperament);

    // println!(
    //     "checking distance of collision={} against threshold={} brake_distance_threshold={}",
    //     distance,
    //     driver_temperament_brake_threshold(&agent.temperament),
    //     brake_distance_threshold
    // );

    // if distance <= brake_distance_threshold {
    //     velocity.y -= CAR_BRAKE_POWER;
    //     velocity.y = f32::max(0., velocity.y);
    // } else {
    let previous_distance = agent.collision_information.last_front_distance;
    let distance_difference = distance - previous_distance;

    let mut velocity_change = CAR_GAS_POWER;

    // println!("if distance={distance} <= brake_distance_threshold={brake_distance_threshold}");

    // brake_distance_threshold is the first point at which cars will start to brake
    // when within brake_distance_threshold AND tail_distance, cars will always brake
    // when within brake_distance_threshold and outside of tail_distance, cars will
    // consider the relative speed of the car ahead: if the car is getting closer, brake
    // if it's pulling away, accelerate

    if distance <= brake_distance_threshold {
        let tail_threshold = driver_temperament_tail_threshold(&agent.temperament);
        let tail_distance = CAR_SIZE.y * tail_threshold;

        // println!("if distance={distance} < tail_distance={tail_distance} OR if distance_difference={distance_difference} > 0. ");

        if distance < tail_distance {
            // car ahead is getting closer; don't accelerate as quickly
            // println!("within tail; braking");
            velocity_change = -CAR_BRAKE_POWER;
        } else if distance_difference > 0. {
            // car ahead is moving away; accelerate a bit faster
            // println!("pulling away; accelerating");
        } else {
            // println!("approaching; braking");
            velocity_change = -CAR_BRAKE_POWER;
        }
    } // else not within braking distance; continue accelerating

    // TODO: all gas power reads should be hidden behind calculator that does agent behavior calcs
    // let direction = Vec2::new(0., distance_difference).normalize();

    // TODO: this should also be encapsulated
    velocity.y += velocity_change;
    velocity.y = f32::max(velocity.y, 0.);

    // println!(
    //     "distance={} previous_distance={} speed change={} new velocity={}",
    //     distance, previous_distance, velocity_change, velocity.y
    // );
}

fn agent_normal_behavior(agent: &mut DriverAgent, velocity: &mut Velocity, transform: &Transform) {
    // if velocity.y < SPEED_LIMIT {
    //     velocity.y += CAR_GAS_POWER;
    // }

    let current_lane_idx = lane_idx_from_screen_pos(&transform.translation.truncate());

    agent.driver_state = DriverState::ChangingLanes;

    // println!(
    //     "current_pos={} current_lane_idx{}",
    //     transform.translation, current_lane_idx
    // );
}

pub fn collision_system(
    mut collider_query: Query<
        (
            Entity,
            &Transform,
            &mut Handle<ColorMaterial>,
            &mut DriverAgent,
            &mut Velocity,
        ),
        With<Car>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>, // for changing car color on collision
) {
    let mut add_intersections: HashMap<Entity, f32> = HashMap::new();
    let mut clear_intersections: HashMap<Entity, f32> = HashMap::new();

    for (entity_1, transform_1, _, _, _) in &collider_query {
        let mut has_collision = false;

        // TODO: don't calculate if agent already has a collision?

        for (entity_2, transform_2, _, _, _) in &collider_query {
            if entity_1 == entity_2 {
                continue;
            }
            // TODO: eliminate double checks by comparing lanes

            // cast ray straight up (y) from transform_1 to intersect with transform_2

            let car_front = get_car_front_middle(transform_1);

            if let Some(intersection) = check_raycast_intersection(
                car_front,
                Direction2d::Y,
                CAR_SIGHT_DISTANCE,
                transform_2,
            ) {
                // println!("Got intersection at {:?}", intersection);

                // guarantee the closest intersection in case there are multiple
                match add_intersections.get_mut(&entity_1) {
                    Some(value) => {
                        if intersection < *value {
                            *value = intersection;
                        }
                    }
                    None => {
                        // doesn't exist yet; insert
                        add_intersections.insert(entity_1, intersection);
                    }
                }

                has_collision = true;
                // TODO: if we could guarantee we saw the closest car (sort by distance?), could break here
                // to prevent further checks
            }
        }

        if !has_collision {
            clear_intersections.insert(entity_1, -1.);
        }
    }

    for (entity_id, intersection_distance) in add_intersections {
        if let Ok(mut entity) = collider_query.get_mut(entity_id) {
            let handle: &Handle<ColorMaterial> = &entity.2;
            if let Some(material) = materials.get_mut(handle) {
                material.color = Color::RED;
            }

            // set previous then current
            entity.3.collision_information.last_front_distance =
                entity.3.collision_information.front_distance;

            entity.3.collision_information.front_distance = intersection_distance;

            // made contact with an object: come to a full stop
            if intersection_distance <= 0. {
                entity.4.x = 0.;
                entity.4.y = 0.;
            }
        }
    }

    for (entity_id, _) in clear_intersections {
        if let Ok(mut entity) = collider_query.get_mut(entity_id) {
            let handle: &Handle<ColorMaterial> = &entity.2;
            if let Some(material) = materials.get_mut(handle) {
                material.color = CAR_COLOR;
            }

            entity.3.collision_information.front_distance = -1.;
        }
    }
}

pub fn check_raycast_intersection(
    origin: Vec2,
    direction: Direction2d,
    max: f32,
    target: &Transform,
) -> Option<f32> {
    let raycast = RayCast2d::new(origin, direction, max);
    let aabb2d = Aabb2d::new(target.translation.truncate(), CAR_SIZE_HALF.truncate());

    raycast.aabb_intersection_at(&aabb2d)
}
