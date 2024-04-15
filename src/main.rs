use bevy::prelude::*;

mod components;
mod constants;
mod stepping;
mod systems;

use crate::components::*;
use crate::constants::*;
use crate::systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .insert_resource(components::Scoreboard { score: 0 })
        .insert_resource(ClearColor(constants::BACKGROUND_COLOR))
        .add_event::<components::CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                systems::keyboard_input_system,
                systems::mouse_click_system,
                systems::cursor_position,
                systems::apply_friction,
                systems::apply_velocity,
                systems::wrap_position,
                //check_for_collisions, play_collision_sound
            )
                .chain(),
        )
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Car
    let car_y = constants::CAR_SPAWN_BOTTOM;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, car_y, 0.0),
                scale: CAR_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: CAR_COLOR,
                ..default()
            },
            ..default()
        },
        Car,
        Collider,
        Velocity(CAR_INITIAL_VELOCITY.normalize() * CAR_SPEED),
        Friction, // TODO: car bundle
    ));

    // components::Scoreboard
    commands.spawn((
        ScoreboardUi,
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // Lanes
    spawn_lanes(&mut commands);
}

fn spawn_lanes(commands: &mut Commands) {
    let total_width = RIGHT_WALL - LEFT_WALL;
    let total_height = TOP_WALL - BOTTOM_WALL;

    let num_lanes: i32 = f32::floor(total_width / LANE_WIDTH) as i32;
    let num_lane_segments: i32 = f32::floor(total_height / LANE_STRIP_SIZE.y) as i32;

    for i in 0..num_lanes {
        let lane_x = lane_to_screen_pos(i + 1).x;

        for j in 0..num_lane_segments {
            if j % 3 != 0 {
                // every third; dotted line
                continue;
            }

            let lane_y = BOTTOM_WALL + LANE_STRIP_SIZE.y * j as f32;

            println!("spawning lane strip at {}:{}", lane_x, lane_y);

            commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(lane_x, lane_y, 0.0),
                    scale: LANE_STRIP_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: STRIPE_COLOR,
                    ..default()
                },
                ..default()
            });
        }
    }
}

// fn check_for_collisions(
//     mut commands: Commands,
// )
