use crate::constants::*;
use bevy::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DebugState {
    #[default]
    Disabled,
    Enabled,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

#[derive(Clone)]
pub enum DriverState {
    Normal,
    ChangingLanes,
}

// axes of DriverAgent behavior:
// lawfulness: likelihood of following rules: keeping to right lane except to pass, percentage of speed limit obeyed
// temperament: acceleration rates, how close to another car they'll get
// patience: willingness to be slowed from their maximum rate (allows a larger slowdown before attempting to pass)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DriverLawfulness {
    Chaotic,
    Orderly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DriverTemperament {
    Psychotic,
    Aggressive,
    Calm,
    Passive,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DriverPatience {
    Enlightened,
    Patient,
    Normal,
    Wild,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LaneChangeDirection {
    Left,
    Right,
    None,
}

lazy_static! {
    pub static ref DRIVER_TEMPERAMENT_TOP_SPEEDS: HashMap<DriverTemperament, f32> = {
        let mut map = HashMap::new();
        map.insert(DriverTemperament::Psychotic, 1.5);
        map.insert(DriverTemperament::Aggressive, 1.3);
        map.insert(DriverTemperament::Calm, 1.0);
        map.insert(DriverTemperament::Passive, 0.8);

        map
    };
    pub static ref DRIVER_PATIENCE_MIN_SPEEDS: HashMap<DriverPatience, f32> = {
        let mut map = HashMap::new();
        map.insert(DriverPatience::Enlightened, 0.2);
        map.insert(DriverPatience::Patient, 0.7);
        map.insert(DriverPatience::Normal, 0.9);
        map.insert(DriverPatience::Wild, 1.0); // always tries to pass

        map
    };
    pub static ref DRIVER_TEMPERAMENT_BRAKE_THRESHOLD: HashMap<DriverTemperament, f32> = {
        // values are percentages of car sight length when a car will start braking
        // e.g., 0.5 means a car won't break until it sees an obstacle within 50% of its sightline.
        // values over 1.0 are ineffective, as a car cannot see farther than its sightline

        let mut map = HashMap::new();
        map.insert(DriverTemperament::Psychotic, 0.5);
        map.insert(DriverTemperament::Aggressive, 0.7);
        map.insert(DriverTemperament::Calm, 1.0);
        map.insert(DriverTemperament::Passive, 1.0);

        map
    };
    pub static ref DRIVER_TEMPERAMENT_TAIL_THRESHOLD: HashMap<DriverTemperament, f32> = {
        // values are percentages of one car length that an agent is willing to tail; e.g., 0.5 means
        // a car will get up to 50% of a car length behind another car,
        // while 3.0 means a car will stay 3 car lengths behind another

        let mut map = HashMap::new();
        map.insert(DriverTemperament::Psychotic, 1.5);
        map.insert(DriverTemperament::Aggressive, 3.0);
        map.insert(DriverTemperament::Calm, 4.5);
        map.insert(DriverTemperament::Passive, 6.0);

        map
    };
}

pub fn driver_temperament_top_speed_pct(temperament: &DriverTemperament) -> f32 {
    DRIVER_TEMPERAMENT_TOP_SPEEDS[&temperament]
}

pub fn driver_temperament_brake_threshold(temperament: &DriverTemperament) -> f32 {
    DRIVER_TEMPERAMENT_BRAKE_THRESHOLD[&temperament]
}

pub fn driver_temperament_tail_threshold(temperament: &DriverTemperament) -> f32 {
    DRIVER_TEMPERAMENT_TAIL_THRESHOLD[&temperament]
}

pub fn driver_patience_min_speed_pct(patience: &DriverPatience) -> f32 {
    DRIVER_PATIENCE_MIN_SPEEDS[&patience]
}

// HashMap::from([(DriverLawfulness::Chaotic, "abc")]);

pub enum WallLocation {
    Left,
    Right,
    Top,
    Bottom,
}

impl WallLocation {
    pub fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
        }
    }

    pub fn size(&self) -> Vec2 {
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

pub fn lane_idx_to_screen_pos(lane_idx: i32) -> Vec3 {
    // given the index of a lane (0 --> num_lanes), return its starting coordinates in screen space
    Vec3::new(LEFT_WALL + LANE_WIDTH * lane_idx as f32, BOTTOM_WALL, 0.0)
}

pub fn lane_idx_from_screen_pos(screen_pos: &Vec2) -> i32 {
    let adjusted_x = screen_pos.x - LEFT_WALL;

    let lane_idx: i32 = f32::floor(adjusted_x / LANE_WIDTH) as i32;

    // println!(
    //     "screen_pos.x={} LEFT_WALL={} adjusted_x={} lane_idx={}",
    //     screen_pos.x, LEFT_WALL, adjusted_x, lane_idx
    // );

    return lane_idx;
}

pub fn lane_idx_to_center(lane_idx: i32) -> Vec3 {
    // given the index of a lane, return the coordinates of that lane's center in screen space
    let lane_pos = lane_idx_to_screen_pos(lane_idx);

    Vec3::new(lane_pos.x + (LANE_WIDTH / 2.), lane_pos.y, 0.0)
}

pub fn cursor_pos_to_screen_space(cursor_pos: &Vec2) -> Vec2 {
    Vec2::new(
        cursor_pos.x - WINDOW_WIDTH_HALF,
        cursor_pos.y - WINDOW_HEIGHT_HALF,
    )
}

pub fn screen_space_to_world_coords(screen_space: &Vec2) -> Vec2 {
    Vec2::new(
        screen_space.x + WINDOW_WIDTH_HALF,
        -screen_space.y + WINDOW_HEIGHT_HALF,
    )
}

pub fn get_car_front_middle(transform: &Transform) -> Vec2 {
    Vec2::new(
        transform.translation.x,
        transform.translation.y + (transform.scale.y / 2.),
    )
}

pub fn get_mouse_text(screen_space: &Vec2, text_coords: &Vec2) -> String {
    format!(
        "World: {}\n\
         UI: {}\n\
         Lane: {}",
        screen_space,
        text_coords,
        lane_idx_from_screen_pos(&screen_space)
    )
}
