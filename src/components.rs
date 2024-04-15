use bevy::prelude::*;

use crate::constants::*;
use crate::util::*;

pub struct CollisionInformation {
    pub front_distance: f32, // -1 if no collision, else distance to closest car in front
}

// COMPONENTS
#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct Collider;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Friction;

#[derive(Component)]
pub struct ScoreboardUi;

#[derive(Component)]
pub struct Lane(pub Vec2);

#[derive(Component)]
pub struct DriverAgent {
    pub driver_state: DriverState,
    pub lane_target: i32,
    pub collision_information: CollisionInformation,
    pub order: DriverOrder,
    pub temperament: DriverTemperament,
    pub patience: DriverPatience,
}

#[derive(Component)]
pub struct LaneChanger;

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
#[derive(Bundle)]
pub struct CarBundle {
    car: Car,
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
            collider: Collider,
            velocity: Velocity(CAR_INITIAL_DIRECTION),
            friction: Friction,
            driver_agent: DriverAgent {
                driver_state: DriverState::Normal,
                lane_target: -1,
                collision_information: CollisionInformation {
                    front_distance: -1.,
                },
                order: DriverOrder::Orderly,
                temperament: DriverTemperament::Calm,
                patience: DriverPatience::Normal,
            },
        }
    }

    pub fn new_with_behavior(
        position: Vec3,
        order: DriverOrder,
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
            collider: Collider,
            velocity: Velocity(CAR_INITIAL_DIRECTION),
            friction: Friction,
            driver_agent: DriverAgent {
                driver_state: DriverState::Normal,
                lane_target: -1,
                collision_information: CollisionInformation {
                    front_distance: -1.,
                },
                order,
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
    order: DriverOrder,
    temperament: DriverTemperament,
    patience: DriverPatience,
) {
    let car_x = lane_idx_to_center(lane_idx).x;
    let car_y = CAR_SPAWN_BOTTOM;
    let car_pos = Vec3::new(car_x, car_y, 0.);

    commands.spawn(CarBundle::new_with_behavior(
        car_pos,
        order,
        temperament,
        patience,
    ));
}
