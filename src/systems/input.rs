use bevy::{prelude::*, window::*};

use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use crate::util::*;

const DIGIT_KEYS: [KeyCode; 10] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Digit0,
];

pub fn check_pause_input(
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    pause_state: &Res<State<PauseState>>,
    next_pause_state: &mut ResMut<NextState<PauseState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        debug!("You pressed P!");
        if *pause_state.get() == PauseState::Paused {
            next_pause_state.set(PauseState::Running);
        } else {
            next_pause_state.set(PauseState::Paused);
        }
    }
}

pub fn digit_input_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if keyboard_input.any_just_pressed(DIGIT_KEYS) {
        for key in keyboard_input.get_just_pressed() {
            if DIGIT_KEYS.contains(key) {
                spawn_car_at_lane(
                    digit_key_to_number(key),
                    &mut commands,
                    meshes.add(Rectangle::default()).into(),
                    materials.add(ColorMaterial::from(Color::PURPLE)),
                    DriverLawfulness::Orderly,
                    DriverTemperament::Calm,
                    DriverPatience::Normal,
                );
            }
        }
    }
}

pub fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &Handle<ColorMaterial>), With<Car>>,
    mut debug_writer: EventWriter<DebugModeEvent>,
    pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    check_pause_input(&keyboard_input, &pause_state, &mut next_pause_state);

    if keyboard_input.just_pressed(KeyCode::Digit0) {
        debug!("YOU PRESSED 0");
    }

    if keyboard_input.just_pressed(KeyCode::KeyD) {
        debug!("You pressed D!");
        debug_writer.send(DebugModeEvent);
    }

    for (mut velocity, material_handle) in &mut query {
        let mut new_color_o: Option<Color> = None;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            debug!("VROOM {}", velocity.y);
            velocity.y += CAR_GAS_POWER;

            new_color_o = Some(Color::GREEN);
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            debug!("SKRRR {}", velocity.y);
            velocity.y -= CAR_BRAKE_POWER;
            velocity.y = f32::max(velocity.y, 0.0);

            new_color_o = Some(Color::RED);
        }

        if let Some(material) = materials.get_mut(material_handle) {
            if let Some(color) = new_color_o {
                material.color = color;
            }
        }
    }
}

pub fn mouse_click_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        // info!("left mouse currently pressed");
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        // info!("left mouse just pressed");
        if let Some(position) = q_windows.single().cursor_position() {
            let adjusted_pos = cursor_pos_to_screen_space(&position);

            let lane_idx = lane_idx_from_screen_pos(&Vec2::from((adjusted_pos.x, adjusted_pos.y)));

            spawn_car_at_lane(
                lane_idx,
                &mut commands,
                meshes.add(Rectangle::default()).into(),
                materials.add(ColorMaterial::from(Color::PURPLE)),
                DriverLawfulness::Orderly,
                DriverTemperament::Calm,
                DriverPatience::Normal,
            );
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        // info!("left mouse just released");
    }
}

fn digit_key_to_number(key: &KeyCode) -> i32 {
    match key {
        KeyCode::Digit1 => 1,
        KeyCode::Digit2 => 2,
        KeyCode::Digit3 => 3,
        KeyCode::Digit4 => 4,
        KeyCode::Digit5 => 5,
        KeyCode::Digit6 => 6,
        KeyCode::Digit7 => 7,
        KeyCode::Digit8 => 8,
        KeyCode::Digit9 => 9,
        KeyCode::Digit0 => 0,
        _ => -1,
    }
}
