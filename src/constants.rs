use bevy::prelude::*;

// WINDOW
pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 720.;
pub const WINDOW_WIDTH_HALF: f32 = WINDOW_WIDTH / 2.;
pub const WINDOW_HEIGHT_HALF: f32 = WINDOW_HEIGHT / 2.;

// CAR
pub const CAR_SPAWN_BOTTOM: f32 = -300.;
pub const CAR_SIZE: Vec3 = Vec3::new(20., 40., 0.);
pub const CAR_INITIAL_DIRECTION: Vec2 = Vec2::new(0., 0.5);
pub const CAR_GAS_POWER: f32 = 10.; // how much velocity the car gains per frame
pub const CAR_BRAKE_POWER: f32 = 12.;
pub const CAR_SIGHT_DISTANCE: f32 = 200.;

pub const LANE_WIDTH: f32 = 40.;
pub const LANE_STRIP_SIZE: Vec3 = Vec3::new(5., 10., 0.);
pub const NUM_LANES: i32 = 2;

// COLORS
pub const BACKGROUND_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
pub const CAR_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
pub const SCORE_COLOR: Color = Color::rgb(1., 0.5, 0.5);
pub const STRIPE_COLOR: Color = Color::WHITE;
pub const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.);
pub const WALL_COLOR: Color = Color::WHITE;

pub const SCOREBOARD_FONT_SIZE: f32 = 40.;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.);

pub const WALL_THICKNESS: f32 = 10.;
// x coordinates
pub const LEFT_WALL: f32 = -450.;
pub const RIGHT_WALL: f32 = 450.;
// y coordinates
pub const BOTTOM_WALL: f32 = -300.;
pub const TOP_WALL: f32 = 300.;

// ENVIRONMENT
pub const FRICTION_DECAY: f32 = 0.996;
pub const SPEED_LIMIT: f32 = 400.;
