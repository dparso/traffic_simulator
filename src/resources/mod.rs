use bevy::math::Vec2;
use bevy::prelude::*;

use crate::components::CarBundle;

#[derive(Resource)]
pub struct CarSpawnRequests {
    pub cars_to_spawn: Vec<CarBundle>,
}

// Store the world position of the mouse cursor
#[derive(Resource, Default)]
pub struct CursorWorldCoords(pub Vec2);

#[derive(Resource)]
pub struct CollisionSound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct Scoreboard {
    pub score: usize,
}
