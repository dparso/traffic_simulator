use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::util::*;

// request that a car be spawned
#[derive(Event)]
pub struct CarSpawnEvent(pub CarBundle);

impl CarSpawnEvent {
    pub fn request_car_spawn(
        car_spawn_events: &mut EventWriter<CarSpawnEvent>,
        lane_idx: i32,
        lawfulness: DriverLawfulness,
        temperament: DriverTemperament,
        patience: DriverPatience,
    ) {
        let car_x = lane_idx_to_center(lane_idx).x;
        let car_y = BOTTOM_WALL + WALL_THICKNESS;
        let car_pos = Vec3::new(car_x, car_y, 0.);
        println!("asked to spawn car at {}", car_pos);

        car_spawn_events.send(CarSpawnEvent(CarBundle::new_with_behavior(
            car_pos,
            lawfulness,
            temperament,
            patience,
        )));
    }

    // TODO: spawn multiple
}
