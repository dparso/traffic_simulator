use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume, RayCast2d},
    prelude::*,
    utils::hashbrown::HashMap,
    window::PrimaryWindow,
};

use crate::components::*;
use crate::constants::*;
use crate::util::*;
use bevy_mod_raycast::prelude::*;

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
            let adjusted_pos = cursor_pos_to_screen_space(position);

            let lane_idx =
                lane_idx_from_screen_pos(Vec3::from((adjusted_pos.x, adjusted_pos.y, 0.0)));

            spawn_car_at_lane(
                lane_idx,
                &mut commands,
                DriverOrder::Orderly,
                DriverTemperament::Psychotic,
                DriverPatience::Wild,
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
        let adjusted_pos = cursor_pos_to_screen_space(position);

        let lane_idx = lane_idx_from_screen_pos(Vec3::from((adjusted_pos.x, adjusted_pos.y, 0.0)));

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

fn agent_accelerate_or_brake(agent: &mut DriverAgent, mut velocity: &mut Velocity) {
    if agent.collision_information.front_distance > -1. {
        brake_for_front(&agent, &mut velocity);
    } else {
        let top_speed = SPEED_LIMIT * driver_temperament_top_speed_pct(&agent.temperament);
        if velocity.y < top_speed {
            velocity.y += CAR_GAS_POWER;
        }
    }
}

fn brake_for_front(agent: &DriverAgent, velocity: &mut Velocity) {
    let distance = agent.collision_information.front_distance;
    let distance_to_brake = CAR_SIZE.y * driver_temperament_distance_threshold(&agent.temperament);

    println!(
        "checking distance of collision={} against threshold={} distance_to_brake={}",
        distance,
        driver_temperament_distance_threshold(&agent.temperament),
        distance_to_brake
    );

    if distance <= distance_to_brake {
        velocity.y -= CAR_BRAKE_POWER;
        velocity.y = f32::max(0., velocity.y);
    }
}

fn agent_normal_behavior(agent: &mut DriverAgent, velocity: &mut Velocity, transform: &Transform) {
    // if velocity.y < SPEED_LIMIT {
    //     velocity.y += CAR_GAS_POWER;
    // }

    let current_lane_idx = lane_idx_from_screen_pos(transform.translation);
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

pub fn collision_system(
    mut collider_query: Query<(Entity, &Transform, &mut Sprite, &mut DriverAgent), With<Car>>,
) {
    let mut add_intersections: HashMap<Entity, f32> = HashMap::new();
    let mut clear_intersections: HashMap<Entity, f32> = HashMap::new();

    for (entity_1, transform_1, _, _) in &collider_query {
        let mut has_collision = false;

        for (entity_2, transform_2, _, _) in &collider_query {
            if entity_1 == entity_2 {
                continue;
            }
            // TODO: eliminate double checks

            // cast ray straight up (y) from transform_1 to intersect with transform_2
            let car_front = get_car_front_middle(transform_1);
            let raycast = RayCast2d::new(car_front, Direction2d::Y, CAR_SIGHT_DISTANCE);

            let box_start = Vec2::new(
                transform_2.translation.x - (transform_2.scale.x / 2.),
                transform_2.translation.y - (transform_2.scale.y / 2.),
            );
            let box_end = Vec2::new(
                transform_2.translation.x + (transform_2.scale.x / 2.),
                transform_2.translation.y + (transform_2.scale.y / 2.),
            );

            let aabb2d = Aabb2d {
                min: box_start,
                max: box_end,
            };

            // println!(
            //     "checking collision with {:?} against box {:?}",
            //     raycast, aabb2d
            // );

            if let Some(intersection) = raycast.aabb_intersection_at(&aabb2d) {
                // println!("Got intersection at {:?}", intersection);
                add_intersections.insert(entity_1, intersection);

                has_collision = true;
                // TODO: if we could guarantee we saw the closest car (sort by distance?), could break here
                // to prevent further checks
            }
        }

        if !has_collision {
            clear_intersections.insert(entity_1, -1.);
        }
    }

    for (entity_id, intersection) in add_intersections {
        if let Ok(mut entity) = collider_query.get_mut(entity_id) {
            entity.2.color = Color::RED;
            entity.3.collision_information.front_distance = intersection;
        }
    }
    for (entity_id, intersection) in clear_intersections {
        if let Ok(mut entity) = collider_query.get_mut(entity_id) {
            entity.2.color = CAR_COLOR;
            entity.3.collision_information.front_distance = -1.;
        }
    }
}
