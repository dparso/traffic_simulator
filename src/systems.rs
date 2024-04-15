use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    window::PrimaryWindow,
};

use crate::components::*;
use crate::constants::*;

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
    mut cars: Query<(&mut Velocity, &mut Sprite), With<Car>>,
) {
    let mut car = cars.single_mut();

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        println!("VROOM {}", car.0.y);
        car.0.y += CAR_GAS_POWER;
        car.1.color = Color::GREEN;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        println!("SKRRR {}", car.0.y);
        car.0.y -= CAR_BRAKE_POWER;
        car.0.y = f32::max(car.0.y, 0.0);

        car.1.color = Color::RED;
    } else {
        car.1.color = CAR_COLOR;
    }
}

pub fn mouse_click_system(mouse_button_input: Res<ButtonInput<MouseButton>>) {
    if mouse_button_input.pressed(MouseButton::Left) {
        info!("left mouse currently pressed");
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        info!("left mouse just pressed");
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        info!("left mouse just released");
    }
}

pub fn cursor_position(q_windows: Query<&Window, With<PrimaryWindow>>) {
    // Games typically only have one window (the primary window)
    if let Some(position) = q_windows.single().cursor_position() {
        println!("Cursor is inside the primary window, at {:?}", position);
    } else {
        println!("Cursor is not in the game window.");
    }
}

// TODO: not a system; where should helper functions live?
pub fn lane_to_screen_pos(lane_idx: i32) -> Vec3 {
    // given the index of a lane (0 --> num_lanes), return its starting coordinates in screen space
    Vec3::new(LEFT_WALL + LANE_WIDTH * lane_idx as f32, BOTTOM_WALL, 0.0)
}
