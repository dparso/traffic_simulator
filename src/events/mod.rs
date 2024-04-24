use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::components::*;

#[derive(Event, Default)]
pub struct CollisionEvent;

// request that a car be spawned
#[derive(Event)]
pub struct CarSpawnEvent(pub CarBundle);

#[derive(Event)]
pub struct LaneChangeEvent();

// impl LaneChangeEvent {

#[derive(Event)]
pub struct DoSomethingComplex(pub Entity, pub f32);

impl From<ListenerInput<Pointer<Down>>> for DoSomethingComplex {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        DoSomethingComplex(event.target, event.hit.depth)
    }
}

// change in debug mode
#[derive(Event)]
pub struct DebugModeEvent;
