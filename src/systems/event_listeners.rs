use bevy::prelude::*;

use crate::util::PauseState;

use crate::{events::DebugModeEvent, DebugMode};

pub fn debug_mode_listener(
    mut events: EventReader<DebugModeEvent>,
    mut debug_mode: ResMut<DebugMode>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    // for _ in events.read() {
    //     debug_mode.0 = !debug_mode.0;
    //     println!(
    //         "Debug Mode is now {}",
    //         if debug_mode.0 { "ON" } else { "OFF" }
    //     );
    // }

    // if debug_mode.0 {
    //     next_pause_state.set(PauseState::Running);
    // } else {
    //     next_pause_state.set(PauseState::Paused);
    // }
}
