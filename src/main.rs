use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::render_resource::ShaderType,
    sprite::MaterialMesh2dBundle,
};

mod stepping;

const CAR_SPAWN_BOTTOM: f32 = -300.0;

const CAR_SIZE: Vec3 = Vec3::new(20.0, 40.0, 0.0);
const CAR_INITIAL_VELOCITY: Vec2 = Vec2::new(0.0, 0.5);
const CAR_SPEED: f32 = 400.0;
const CAR_GAS_POWER: f32 = 10.0; // how much velocity the car gains per frame
const CAR_BRAKE_POWER: f32 = 12.0;

const FRICTION_DECAY: f32 = 0.996;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const CAR_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

// COMPONENTS
#[derive(Component)]
struct Car;

#[derive(Component)]
struct Collider;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Friction;

#[derive(Component)]
struct ScoreboardUi;

// RESOURCES
#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

// EVENTS
#[derive(Event, Default)]
struct CollisionEvent;

// BUNDLES
#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum WallLocation {
    Left,
    Right,
    Top,
    Bottom,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;

        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Top | WallLocation::Bottom => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                take_input,
                apply_friction,
                apply_velocity,
                wrap_position,
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
    let car_y = CAR_SPAWN_BOTTOM;

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

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
}

fn apply_friction(mut query: Query<&mut Velocity, With<Friction>>) {
    for mut velocity in &mut query {
        println!(
            "velocity.y={} *= {} = {}",
            velocity.y,
            FRICTION_DECAY,
            velocity.y * FRICTION_DECAY
        );

        velocity.x *= FRICTION_DECAY;
        velocity.y *= FRICTION_DECAY;

        println!("Friction slowed me down to {:?}", velocity.0);
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn wrap_position(mut query: Query<&mut Transform, With<Velocity>>) {
    for mut transform in &mut query {
        if transform.translation.y > TOP_WALL - (transform.scale.y / 2.0) {
            transform.translation.y = BOTTOM_WALL + (transform.scale.y / 2.0);
        }
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreboardUi>>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn take_input(
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

// fn check_for_collisions(
//     mut commands: Commands,
// )
