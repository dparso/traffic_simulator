use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::*};
use bevy_egui::{
    egui::{self, ScrollArea},
    EguiContexts, EguiPlugin,
};
use bevy_mod_picking::prelude::*;

pub mod components;
pub mod constants;
pub mod events;
pub mod resources;
pub mod stepping;
pub mod systems;
pub mod util;

use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::resources::*;
use crate::util::*;

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos {}

fn main() {
    App::new()
        /////////////
        // PLUGINS //
        /////////////
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT)
                        .with_scale_factor_override(1.),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin,
            DefaultPickingPlugins,
        ))
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.), Val::Percent(50.)),
        )
        .init_gizmo_group::<MyRoundGizmos>()
        ///////////////
        // RESOURCES //
        ///////////////
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CarSpawnRequests {
            cars_to_spawn: vec![],
        })
        .insert_resource(DebugMode(false))
        .init_resource::<CursorWorldCoords>()
        ////////////
        // STATES //
        ////////////
        .init_state::<PauseState>()
        ////////////
        // EVENTS //
        ////////////
        .add_event::<events::CollisionEvent>()
        .add_event::<events::CarSpawnEvent>()
        .add_event::<events::DebugModeEvent>()
        .add_event::<DoSomethingComplex>()
        /////////////
        // SYSTEMS //
        /////////////
        // .configure_sets(Update, (SomeSet.run_if(in_state(PauseState::Paused))))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                systems::collision_system,
                systems::apply_friction,
                systems::apply_velocity,
                systems::wrap_position,
                systems::agent_check_lane_change_system,
                systems::agent_active_lane_change_system,
                systems::agent_drive_system,
            )
                .run_if(in_state(PauseState::Running))
                .chain(),
        )
        .add_systems(
            Update,
            (
                something_complex,
                ui_example,
                bevy::window::close_on_esc,
                systems::cursor_system,
                systems::debug_mouse_system,
                systems::keyboard_input_system,
                systems::digit_input_system,
                // systems::mouse_click_system,
                systems::draw_car_sight_lines,
                systems::debug_mode_listener,
                // update_scoreboard,
                // draw_example_collection,
                // update_config,
                // receive_greetings.run_if(on_event::<DoSomethingComplex>()),
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
    commands.spawn((Camera2dBundle::default(), MainCamera));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::default()).into(),
            transform: Transform::default().with_scale(Vec3::splat(128.)),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            ..default()
        },
        PickableBundle::default(),
        On::<Pointer<Down>>::send_event::<DoSomethingComplex>(),
    ));

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Cars
    spawn_car_at_lane(
        0,
        &mut commands,
        meshes.add(Rectangle::default()).into(),
        materials.add(ColorMaterial::from(CAR_COLOR)),
        DriverLawfulness::Orderly,
        DriverTemperament::Passive,
        DriverPatience::Normal,
    );

    spawn_car_at_lane(
        1,
        &mut commands,
        meshes.add(Rectangle::default()).into(),
        materials.add(ColorMaterial::from(CAR_COLOR)),
        DriverLawfulness::Orderly,
        DriverTemperament::Passive,
        DriverPatience::Normal,
    );

    // Scoreboard
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

    // Mouse text
    commands.spawn((
        MouseText,
        TextBundle::from_section(
            get_mouse_text(&Vec2::splat(0.), &Vec2::splat(0.)),
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 12.,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(-WINDOW_HEIGHT), // hidden until mouse in view
            left: Val::Px(-WINDOW_WIDTH),
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
    // let total_width = RIGHT_WALL - LEFT_WALL;
    let total_height = TOP_WALL - BOTTOM_WALL;

    // let num_lanes: i32 = f32::floor(total_width / LANE_WIDTH) as i32;
    let num_lane_segments: i32 = f32::floor(total_height / LANE_STRIP_SIZE.y) as i32;

    for i in 0..NUM_LANES {
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
    cursor_coords: ResMut<CursorWorldCoords>,
) {
    // let sin = time.elapsed_seconds().sin() * 50.;
    // gizmos.line_2d(Vec2::Y * -sin, Vec2::splat(-80.), Color::RED);
    // gizmos.ray_2d(Vec2::Y * sin, Vec2::splat(80.), Color::GREEN);

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

    gizmos.line_2d(
        Vec2::new(WINDOW_WIDTH_HALF, WINDOW_HEIGHT_HALF),
        Vec2::new(cursor_coords.0.x, cursor_coords.0.y), // y is inverted for mouse position...
        Color::RED,
    );

    // let bottom_left = Vec2::new(0., -CAR_SIZE_HALF.y);
    // let top_right = Vec2::new(LANE_WIDTH_DOUBLE, CAR_SIZE_HALF.y);

    // let bounding_box = Aabb2d {
    //     min: Vec2::new(-50., -50.),
    //     max: Vec2::new(50., 50.),
    // };

    // let intersect_box = Aabb2d {
    //     min: Vec2::new(cursor_coords.0.x - 25., cursor_coords.0.y - 25.),
    //     max: Vec2::new(cursor_coords.0.x + 25., cursor_coords.0.y + 25.),
    // };

    // gizmos.rect_2d(Vec2::ZERO, 0., Vec2::splat(100.), Color::BLACK);
    // gizmos.rect_2d(cursor_coords.0, 0., Vec2::splat(50.), Color::BLACK);
    // gizmos.rect_2d(Vec2::new(), 0., Vec2::splat(20.), Color::BLACK);

    // gizmos.rect_2d(bottom_left, 0., bounding_box.half_size() * 2., Color::BLACK);
    // gizmos.rect_2d(
    //     bottom_left / 2.,
    //     0.,
    //     intersect_box.half_size() * 2.,
    //     Color::BLACK,
    // );

    // println!("bounding={:?} intersect={:?}", bounding_box, intersect_box);

    // if bounding_box.intersects(&intersect_box) {
    //     println!("YES intersection");
    // } else {
    //     println!("NO intersection");
    // }
    // if bounding_box.contains(&intersect_box) {
    //     println!("YES contains");
    // } else {
    //     println!("NO contains");
    // }

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

fn something_complex(
    mut reader: EventReader<DoSomethingComplex>,
    world: &World,
    mut commands: Commands,
) {
    for event in reader.read() {
        // commands.entity(event.0).
        info!(
            "Hello {:?}, you are {:?} depth units away from the pointer",
            event.0, event.1
        );

        // if let Some(ent) = world.get_entity_mut(event.0) {
        let components_in_entity = world.inspect_entity(event.0);
        info!("Got {} components", components_in_entity.len());

        for component in components_in_entity {
            info!("Entity {:?} has components {:?}", event.0, component);
        }
    }
}
fn ui_example(mut egui_contexts: EguiContexts, mut number: Local<f32>) {
    egui::SidePanel::left("Left").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.heading("Note that while a slider is being dragged, the panel is being resized, or the scrollbar is being moved, items in the 3d scene cannot be picked even if the mouse is over them.");
                for _ in 0..100 {
                    ui.add(egui::Slider::new(&mut *number, 0.0..=100.0));
                }
            })
    });
    egui::Window::new("Demo").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
            ui.heading("Note that you can select a 3d object then click on this egui window without that object being deselected!");
        });
    });
}
