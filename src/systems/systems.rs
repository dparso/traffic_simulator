use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume, RayCast2d},
    prelude::*,
    utils::hashbrown::HashMap,
    window::PrimaryWindow,
};

use crate::components::*;
use crate::constants::Direction;
use crate::constants::*;
use crate::util::*;

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

pub fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Sprite), With<Car>>,
) {
    for (mut velocity, mut sprite) in &mut query {
        let initial_color = sprite.color;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            println!("VROOM {}", velocity.y);
            velocity.y += CAR_GAS_POWER;
            sprite.color = Color::GREEN;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            println!("SKRRR {}", velocity.y);
            velocity.y -= CAR_BRAKE_POWER;
            velocity.y = f32::max(velocity.y, 0.0);

            sprite.color = Color::RED;
        } else {
            sprite.color = initial_color;
        }
    }
}

pub fn mouse_click_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        info!("left mouse currently pressed");
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        info!("left mouse just pressed");
        if let Some(position) = q_windows.single().cursor_position() {
            let adjusted_pos = cursor_pos_to_screen_space(&position);

            let lane_idx =
                lane_idx_from_screen_pos(&Vec3::from((adjusted_pos.x, adjusted_pos.y, 0.0)));

            spawn_car_at_lane(
                lane_idx,
                &mut commands,
                DriverLawfulness::Orderly,
                DriverTemperament::Calm,
                DriverPatience::Normal,
            );
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        info!("left mouse just released");
    }
}

pub fn cursor_position(q_windows: Query<&Window, With<PrimaryWindow>>) {
    // Games typically only have one window (the primary window)
    if let Some(position) = q_windows.single().cursor_position() {
        let adjusted_pos = cursor_pos_to_screen_space(&position);

        let lane_idx = lane_idx_from_screen_pos(&Vec3::from((adjusted_pos.x, adjusted_pos.y, 0.0)));

        println!(
            "Cursor is inside the primary window, at logical={:?}, adjusted={}, lane={}",
            position, adjusted_pos, lane_idx
        );
    } else {
        // println!("Cursor is not in the game window.");
    }
}

pub fn driver_agent_system(mut query: Query<(&mut DriverAgent, &mut Velocity, &Transform)>) {
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

pub fn agent_lane_change_system(
    mut query: Query<(Entity, &mut DriverAgent, &mut Velocity, &Transform)>,
) {
    for (entity_id_1, agent_1, velocity, transform_1) in &query {
        if should_change_lanes(&agent_1, &velocity) {
            // ??
            // attempt_lane_move(&Direction::Right, &mut velocity, &transform);
        }

        for (entity_id_2, agent_2, _, transform_2) in &query {
            if entity_id_1 == entity_id_2 {
                continue;
            }
        }
    }
}

fn should_change_lanes(agent: &DriverAgent, velocity: &Velocity) -> bool {
    // drivers want to change lanes in two scenarios:
    //  there's a car in front of them, and they're impatient
    //  they're law-abiding and want to move to the right lane when not passing

    let min_speed_threshold = driver_patience_min_speed_pct(&agent.patience);

    if has_obstacle_in_range(&agent) && velocity.y < min_speed_threshold {
        println!(
            "I want to pass you! velocity={} threshold={}",
            velocity.y, min_speed_threshold
        );
        return true;
    } else {
        match agent.lawfulness {
            // TODO: orderly checks right side
            DriverLawfulness::Chaotic => {
                return false;
            }
            DriverLawfulness::Orderly => {}
        }
    }

    return false;
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

fn attempt_lane_move(
    direction: &crate::constants::Direction,
    mut velocity: &mut Velocity,
    transform: &Transform,
) {
    let current_lane = lane_idx_from_screen_pos(&transform.translation);

    match direction {
        Direction::Left => if is_lane_open(current_lane, current_lane + 1, &transform) {},
        Direction::Right => {}
        _ => {}
    }
}

fn has_obstacle_in_range(agent: &DriverAgent) -> bool {
    agent.collision_information.front_distance > -1.
}

fn is_lane_open(from_lane: i32, to_lane: i32, transform: &Transform) -> bool {
    // cast rays from front and back of car in the direction of target lane

    let top_middle = Vec2::new(
        transform.translation.x,
        transform.translation.y + (CAR_SIZE.y / 2.),
    );

    let bottom_middle = Vec2::new(
        transform.translation.x,
        transform.translation.y - (CAR_SIZE.y / 2.),
    );

    let look_direction = if to_lane - from_lane > 0 {
        Direction2d::X
    } else {
        Direction2d::NEG_X
    };

    if let Some(intersection) = check_intersection(
        top_middle,
        look_direction,
        CAR_SIDE_CHECK_DISTANCE,
        transform,
    ) {}

    if let Some(intersection) = check_intersection(
        bottom_middle,
        look_direction,
        CAR_SIDE_CHECK_DISTANCE,
        transform,
    ) {}

    true
}

fn brake_for_front(agent: &DriverAgent, velocity: &mut Velocity) {
    let distance = agent.collision_information.front_distance;
    let brake_distance_threshold =
        CAR_SIGHT_DISTANCE * driver_temperament_brake_threshold(&agent.temperament);

    println!(
        "checking distance of collision={} against threshold={} brake_distance_threshold={}",
        distance,
        driver_temperament_brake_threshold(&agent.temperament),
        brake_distance_threshold
    );

    // if distance <= brake_distance_threshold {
    //     velocity.y -= CAR_BRAKE_POWER;
    //     velocity.y = f32::max(0., velocity.y);
    // } else {
    let previous_distance = agent.collision_information.last_front_distance;
    let distance_difference = distance - previous_distance;

    let mut velocity_change = CAR_GAS_POWER;

    println!("if distance={distance} <= brake_distance_threshold={brake_distance_threshold}");

    // brake_distance_threshold is the first point at which cars will start to brake
    // when within brake_distance_threshold AND tail_distance, cars will always brake
    // when within brake_distance_threshold and outside of tail_distance, cars will
    // consider the relative speed of the car ahead: if the car is getting closer, brake
    // if it's pulling away, accelerate

    if distance <= brake_distance_threshold {
        let tail_threshold = driver_temperament_tail_threshold(&agent.temperament);
        let tail_distance = CAR_SIZE.y * tail_threshold;

        println!("if distance={distance} < tail_distance={tail_distance} OR if distance_difference={distance_difference} > 0. ");

        if distance < tail_distance {
            // car ahead is getting closer; don't accelerate as quickly
            println!("within tail; braking");
            velocity_change = -CAR_BRAKE_POWER;
        } else if distance_difference > 0. {
            // car ahead is moving away; accelerate a bit faster
            println!("pulling away; accelerating");
        } else {
            println!("approaching; braking");
            velocity_change = -CAR_BRAKE_POWER;
        }
    } // else not within braking distance; continue accelerating

    // TODO: all gas power reads should be hidden behind calculator that does agent behavior calcs
    // let direction = Vec2::new(0., distance_difference).normalize();

    // TODO: this should also be encapsulated
    velocity.y += velocity_change;
    velocity.y = f32::max(velocity.y, 0.);

    println!(
        "distance={} previous_distance={} speed change={} new velocity={}",
        distance, previous_distance, velocity_change, velocity.y
    );
}

fn agent_normal_behavior(agent: &mut DriverAgent, velocity: &mut Velocity, transform: &Transform) {
    // if velocity.y < SPEED_LIMIT {
    //     velocity.y += CAR_GAS_POWER;
    // }

    let current_lane_idx = lane_idx_from_screen_pos(&transform.translation);
    let next_lane_idx = current_lane_idx + 1;

    agent.driver_state = DriverState::ChangingLanes;
    agent.lane_target = next_lane_idx;

    println!(
        "current_pos={} current_lane_idx={} next_lane_idx={} target={}",
        transform.translation, current_lane_idx, next_lane_idx, agent.lane_target
    );
}

fn agent_changing_lanes_behavior(
    agent: &mut DriverAgent,
    velocity: &mut Velocity,
    transform: &Transform,
) {
    // TODO: negative velocity
    // let current_lane_idx = lane_idx_from_screen_pos(transform.translation);
    let target_lane_center = lane_idx_to_center(agent.lane_target);
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
    }
}

pub fn check_intersection(
    origin: Vec2,
    direction: Direction2d,
    max: f32,
    target: &Transform,
) -> Option<f32> {
    let raycast = RayCast2d::new(origin, direction, max);

    let box_start = Vec2::new(
        target.translation.x - (target.scale.x / 2.),
        target.translation.y - (target.scale.y / 2.),
    );
    let box_end = Vec2::new(
        target.translation.x + (target.scale.x / 2.),
        target.translation.y + (target.scale.y / 2.),
    );

    let aabb2d = Aabb2d {
        min: box_start,
        max: box_end,
    };

    raycast.aabb_intersection_at(&aabb2d)
}

pub fn collision_system(
    mut collider_query: Query<
        (
            Entity,
            &Transform,
            &mut Sprite,
            &mut DriverAgent,
            &mut Velocity,
        ),
        With<Car>,
    >,
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
            // TODO: eliminate double checks

            // cast ray straight up (y) from transform_1 to intersect with transform_2

            let car_front = get_car_front_middle(transform_1);

            if let Some(intersection) =
                check_intersection(car_front, Direction2d::Y, CAR_SIGHT_DISTANCE, transform_2)
            {
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
            entity.2.color = Color::RED;

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
            entity.2.color = CAR_COLOR;
            entity.3.collision_information.front_distance = -1.;
        }
    }
}
