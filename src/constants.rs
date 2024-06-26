use bevy::prelude::*;

// WINDOW
pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 1440.;
pub const WINDOW_WIDTH_HALF: f32 = WINDOW_WIDTH / 2.;
pub const WINDOW_HEIGHT_HALF: f32 = WINDOW_HEIGHT / 2.;

// CAR
pub const CAR_SIZE: Vec3 = Vec3::new(20., 40., 1.); // note: a non-zero Z size is necessary for picker (won't detect mouse hover)
pub const CAR_SIZE_HALF: Vec3 = Vec3::new(CAR_SIZE.x / 2., CAR_SIZE.y / 2., CAR_SIZE.z / 2.);
pub const CAR_INITIAL_DIRECTION: Vec2 = Vec2::new(0., 0.5);
pub const CAR_GAS_POWER: f32 = 10.; // how much velocity the car gains per frame
pub const CAR_BRAKE_POWER: f32 = 15.;
pub const CAR_SIGHT_DISTANCE: f32 = 300.;

// how far to either side of the car will be checked when attempting to change lanes
pub const CAR_SIDE_CHECK_DISTANCE: f32 = LANE_WIDTH + (CAR_SIZE.y / 2.);

pub const LANE_WIDTH: f32 = 40.;
pub const LANE_WIDTH_DOUBLE: f32 = LANE_WIDTH * 2.;
pub const LANE_STRIP_SIZE: Vec3 = Vec3::new(5., 10., 0.);
pub const NUM_LANES: i32 = 2;

// COLORS
pub const BACKGROUND_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
pub const CAR_COLOR: Color = Color::MIDNIGHT_BLUE;
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
pub const BOTTOM_WALL: f32 = -600.;
pub const TOP_WALL: f32 = 600.;

// ENVIRONMENT
pub const FRICTION_DECAY: f32 = 0.996;
pub const SPEED_LIMIT: f32 = 200.;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
