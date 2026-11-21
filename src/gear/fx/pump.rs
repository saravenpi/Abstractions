//! The sound of the boot water pump: a pulse at a rate that grows with the spool, from the body
//! it is strapped to, which carries it to the ears even in a vacuum.

use bevy::prelude::*;

use super::super::Gear;
use super::super::jet::jetting;
use super::super::tune::PUMP_RATE;
use crate::audio::spatial::tune::PUMP;
use crate::audio::{AudioCue, Origin, Sound};
use crate::player::{Player, local};
use crate::sim::PlayerId;

/// Sends a pump cue at a rate from [`PUMP_RATE`] while the boots of the local player jet.
pub fn pump(
    time: Res<Time>,
    players: Query<(&PlayerId, &Gear, &GlobalTransform), With<Player>>,
    mut due: Local<f32>,
    mut sounds: MessageWriter<Sound>,
) {
    let worn = local(
        players
            .iter()
            .map(|(id, gear, at)| (*id, (gear, at.translation()))),
    );
    let Some((gear, at)) = worn.filter(|(gear, _)| jetting(&gear.input)) else {
        *due = 0.0;
        return;
    };
    let spool = gear.state.power();
    *due += time.delta_secs() * (PUMP_RATE.0 + (PUMP_RATE.1 - PUMP_RATE.0) * spool);
    if *due >= 1.0 {
        *due %= 1.0;
        sounds.write(Sound {
            cue: AudioCue::Pump(spool),
            from: Origin::Held(at, PUMP.0, PUMP.1),
        });
    }
}
