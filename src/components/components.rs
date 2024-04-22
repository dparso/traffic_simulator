use bevy::prelude::*;

use crate::constants::*;
use crate::util::*;

#[derive(Clone)]
pub struct CollisionInformation {
    pub front_distance: f32, // -1 if no collision, else distance to closest car in front
    pub last_front_distance: f32, // previous frame's value of front_distance
}

// COMPONENTS

/// Used to help identify main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Clone)]
pub struct Car;

// an entity that has a position in a certain lane
#[derive(Component, Clone)]
pub struct LaneEntity(pub i32);

#[derive(Component, Clone)]
pub struct Collider;

#[derive(Component, Clone, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone)]
pub struct Friction;

#[derive(Component)]
pub struct ScoreboardUi;

#[derive(Component)]
pub struct Lane(pub Vec2);

#[derive(Component, Clone)]
pub struct DriverAgent {
    pub driver_state: DriverState,
    pub collision_information: CollisionInformation,
    pub lawfulness: DriverLawfulness,
    pub temperament: DriverTemperament,
    pub patience: DriverPatience,
}

#[derive(Component)]
pub struct LaneChanger;

#[derive(Component)]
pub struct ActiveLaneChange {
    pub lane_change_direction: LaneChangeDirection,
    pub lane_target: i32,
}

// RESOURCES
#[derive(Resource)]
pub struct CollisionSound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct Scoreboard {
    pub score: usize,
}

// EVENTS
#[derive(Event, Default)]
pub struct CollisionEvent;

// BUNDLES
#[derive(Bundle, Clone)]
pub struct CarBundle {
    car: Car,
    lane: LaneEntity,
    sprite_bundle: SpriteBundle,
    collider: Collider,
    velocity: Velocity,
    friction: Friction,
    driver_agent: DriverAgent,
}

impl CarBundle {
    pub fn new(position: Vec3) -> CarBundle {
        CarBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position,
                    scale: CAR_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: CAR_COLOR,
                    ..default()
                },
                ..default()
            },
            car: Car,
            lane: LaneEntity(lane_idx_from_screen_pos(&position)),
            collider: Collider,
            velocity: Velocity(CAR_INITIAL_DIRECTION),
            friction: Friction,
            driver_agent: DriverAgent {
                driver_state: DriverState::Normal,
                collision_information: CollisionInformation {
                    front_distance: -1.,
                    last_front_distance: -1.,
                },
                lawfulness: DriverLawfulness::Orderly,
                temperament: DriverTemperament::Calm,
                patience: DriverPatience::Normal,
            },
        }
    }

    pub fn new_with_behavior(
        position: Vec3,
        lawfulness: DriverLawfulness,
        temperament: DriverTemperament,
        patience: DriverPatience,
    ) -> CarBundle {
        CarBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position,
                    scale: CAR_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: CAR_COLOR,
                    ..default()
                },
                ..default()
            },
            car: Car,
            lane: LaneEntity(lane_idx_from_screen_pos(&position)),
            collider: Collider,
            velocity: Velocity(CAR_INITIAL_DIRECTION * SPEED_LIMIT),
            friction: Friction,
            driver_agent: DriverAgent {
                driver_state: DriverState::Normal,
                collision_information: CollisionInformation {
                    front_distance: -1.,
                    last_front_distance: -1.,
                },
                lawfulness,
                temperament,
                patience,
            },
        }
    }
}

#[derive(Bundle)]
pub struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl WallBundle {
    pub fn new(location: WallLocation) -> WallBundle {
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

pub fn spawn_car_at_lane(
    lane_idx: i32,
    commands: &mut Commands,
    lawfulness: DriverLawfulness,
    temperament: DriverTemperament,
    patience: DriverPatience,
) {
    let car_x = lane_idx_to_center(lane_idx).x;
    let car_y = BOTTOM_WALL + WALL_THICKNESS;
    let car_pos = Vec3::new(car_x, car_y, 0.);

    commands.spawn(CarBundle::new_with_behavior(
        car_pos,
        lawfulness,
        temperament,
        patience,
    ));
}
