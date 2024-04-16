use bevy::{prelude::*, window::*};

mod components;
mod constants;
mod stepping;
mod systems;
mod util;

use crate::components::*;
use crate::constants::*;
use crate::systems::*;
use crate::util::*;
use bevy_mod_raycast::prelude::*;
use std::f32::consts::{PI, TAU};

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos {}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT)
                        .with_scale_factor_override(1.),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.), Val::Percent(50.)),
        )
        .init_gizmo_group::<MyRoundGizmos>()
        .insert_resource(components::Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<components::CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                systems::keyboard_input_system,
                systems::mouse_click_system,
                systems::cursor_position,
                systems::collision_system,
                systems::apply_friction,
                systems::apply_velocity,
                systems::wrap_position,
                systems::driver_agent_system,
                // systems::raycast_system,
                //check_for_collisions, play_collision_sound
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_scoreboard,
                // draw_example_collection,
                draw_car_sight_lines,
                update_config,
                bevy::window::close_on_esc,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera

    // commands.spawn((Camera2dBundle::default(), RaycastSource::<()>::new_cursor()));
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         mesh: meshes.add(Circle::default()).into(),
    //         transform: Transform::default().with_scale(Vec3::splat(128.)),
    //         material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //         ..default()
    //     },
    //     RaycastMesh::<()>::default(), // Make this mesh ray cast-able;
    // ));

    commands.spawn(Camera2dBundle::default());

    // commands.spawn(Raycast());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Car
    spawn_car_at_lane(
        0,
        &mut commands,
        DriverOrder::Orderly,
        DriverTemperament::Passive,
        DriverPatience::Normal,
    );
    spawn_car_at_lane(
        1,
        &mut commands,
        DriverOrder::Orderly,
        DriverTemperament::Passive,
        DriverPatience::Normal,
    );

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

    // text
    commands.spawn(TextBundle::from_section(
        "Hold 'Left' or 'Right' to change the line width of straight gizmos\n\
            Hold 'Up' or 'Down' to change the line width of round gizmos\n\
            Press '1' or '2' to toggle the visibility of straight gizmos or round gizmos",
        TextStyle {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 24.,
            color: Color::WHITE,
        },
    ));
}

fn spawn_lanes(commands: &mut Commands) {
    let total_width = RIGHT_WALL - LEFT_WALL;
    let total_height = TOP_WALL - BOTTOM_WALL;

    let num_lanes: i32 = f32::floor(total_width / LANE_WIDTH) as i32;
    let num_lane_segments: i32 = f32::floor(total_height / LANE_STRIP_SIZE.y) as i32;

    for i in 0..num_lanes {
        let lane_x = lane_idx_to_screen_pos(i + 1).x;

        for j in 0..num_lane_segments {
            if j % 3 != 0 {
                // every third; dotted line
                continue;
            }

            let lane_y = BOTTOM_WALL + LANE_STRIP_SIZE.y * j as f32;

            commands.spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(lane_x, lane_y, 0.),
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

fn draw_example_collection(
    mut gizmos: Gizmos,
    mut my_gizmos: Gizmos<MyRoundGizmos>,
    time: Res<Time>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let sin = time.elapsed_seconds().sin() * 50.;
    gizmos.line_2d(Vec2::Y * -sin, Vec2::splat(-80.), Color::RED);
    gizmos.ray_2d(Vec2::Y * sin, Vec2::splat(80.), Color::GREEN);

    // Triangle
    // gizmos.linestrip_gradient_2d([
    //     (Vec2::Y * 300., Color::BLUE),
    //     (Vec2::new(-255., -155.), Color::RED),
    //     (Vec2::new(255., -155.), Color::GREEN),
    //     (Vec2::Y * 300., Color::BLUE),
    // ]);

    gizmos.line_2d(
        Vec2::ZERO,
        Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        Color::GREEN,
    );

    if let Some(position) = q_windows.single().cursor_position() {
        let adjusted_x = position.x - WINDOW_WIDTH_HALF;
        let adjusted_y = position.y - WINDOW_HEIGHT_HALF;

        gizmos.line_2d(
            Vec2::new(adjusted_x, -adjusted_y), // y is inverted for mouse position...
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            Color::RED,
        );
    }

    // gizmos.rect_2d(
    //     Vec2::ZERO,
    //     time.elapsed_seconds() / 3.,
    //     Vec2::splat(300.),
    //     Color::BLACK,
    // );

    // // The circles have 32 line-segments by default.
    // my_gizmos.circle_2d(Vec2::ZERO, 120., Color::BLACK);
    // my_gizmos.ellipse_2d(
    //     Vec2::ZERO,
    //     time.elapsed_seconds() % TAU,
    //     Vec2::new(100., 200.),
    //     Color::YELLOW_GREEN,
    // );
    // // You may want to increase this for larger circles.
    // my_gizmos
    //     .circle_2d(Vec2::ZERO, 300., Color::NAVY)
    //     .segments(64);

    // // Arcs default amount of segments is linearly interpolated between
    // // 1 and 32, using the arc length as scalar.
    // my_gizmos.arc_2d(Vec2::ZERO, sin / 10., PI / 2., 350., Color::ORANGE_RED);

    // gizmos.arrow_2d(
    //     Vec2::ZERO,
    //     Vec2::from_angle(sin / -10. + PI / 2.) * 50.,
    //     Color::YELLOW,
    // );
}

fn update_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    if keyboard.pressed(KeyCode::ArrowRight) {
        config.line_width += 5. * time.delta_seconds();
        config.line_width = config.line_width.clamp(0., 50.);
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        config.line_width -= 5. * time.delta_seconds();
        config.line_width = config.line_width.clamp(0., 50.);
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        config.enabled ^= true;
    }

    let (my_config, _) = config_store.config_mut::<MyRoundGizmos>();
    if keyboard.pressed(KeyCode::ArrowUp) {
        my_config.line_width += 5. * time.delta_seconds();
        my_config.line_width = my_config.line_width.clamp(0., 50.);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        my_config.line_width -= 5. * time.delta_seconds();
        my_config.line_width = my_config.line_width.clamp(0., 50.);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        my_config.enabled ^= true;
    }
}

fn draw_car_sight_lines(query: Query<&Transform, With<Car>>, mut gizmos: Gizmos) {
    for transform in &query {
        let line_start = get_car_front_middle(transform);

        gizmos.ray_2d(
            line_start,
            Vec2::new(0., 1.) * CAR_SIGHT_DISTANCE,
            Color::GREEN,
        );
    }
}

// fn check_for_collisions(
//     mut commands: Commands,
// )
