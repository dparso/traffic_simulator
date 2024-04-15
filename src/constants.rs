use bevy::prelude::*;

pub const CAR_SPAWN_BOTTOM: f32 = -300.0;

pub const CAR_SIZE: Vec3 = Vec3::new(20.0, 40.0, 0.0);
pub const CAR_INITIAL_VELOCITY: Vec2 = Vec2::new(0.0, 0.5);
pub const CAR_SPEED: f32 = 400.0;
pub const CAR_GAS_POWER: f32 = 10.0; // how much velocity the car gains per frame
pub const CAR_BRAKE_POWER: f32 = 12.0;

pub const FRICTION_DECAY: f32 = 0.996;

pub const LANE_WIDTH: f32 = 40.0;
pub const LANE_STRIP_SIZE: Vec3 = Vec3::new(5.0, 10.0, 0.0);
pub const NUM_LANES: i32 = 2;

pub const BACKGROUND_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
pub const CAR_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
pub const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
pub const STRIPE_COLOR: Color = Color::WHITE;
pub const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
pub const WALL_COLOR: Color = Color::WHITE;

pub const SCOREBOARD_FONT_SIZE: f32 = 40.0;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

pub const WALL_THICKNESS: f32 = 10.0;
// x coordinates
pub const LEFT_WALL: f32 = -450.;
pub const RIGHT_WALL: f32 = 450.;
// y coordinates
pub const BOTTOM_WALL: f32 = -300.;
pub const TOP_WALL: f32 = 300.;
