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
pub struct SelectEntityEvent(pub Entity);

impl From<ListenerInput<Pointer<Select>>> for SelectEntityEvent {
    fn from(event: ListenerInput<Pointer<Select>>) -> Self {
        SelectEntityEvent(event.target)
    }
}

#[derive(Event)]
pub struct DeselectEntityEvent(pub Entity);

impl From<ListenerInput<Pointer<Deselect>>> for DeselectEntityEvent {
    fn from(event: ListenerInput<Pointer<Deselect>>) -> Self {
        DeselectEntityEvent(event.target)
    }
}

// requests to modify an entity with `SelectedEntity` component
// by replacing its `DriverAgent` component with the provided
#[derive(Event)]
pub struct ModifySelectedDriverAgentEvent(pub DriverAgent);
