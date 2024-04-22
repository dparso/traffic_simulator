use bevy::prelude::Resource;

use crate::components::CarBundle;

#[derive(Resource)]
pub struct CarSpawnRequests {
    pub cars_to_spawn: Vec<CarBundle>,
}
