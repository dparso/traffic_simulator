use bevy::prelude::*;

use crate::events::CarSpawnEvent;

pub fn car_spawn_system(mut commands: Commands, mut events: EventReader<CarSpawnEvent>) {
    for event in events.read() {
        println!("reading event to spawn");
        commands.spawn(event.0.clone());
    }
}
