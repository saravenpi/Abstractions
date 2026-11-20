//! The sound of the boot water pump: a pulse at a rate that grows with the spool.

use bevy::prelude::*;

use super::super::Gear;
use super::super::jet::jetting;
use super::super::tune::PUMP_RATE;
use crate::audio::AudioCue;
use crate::player::{Player, local};
use crate::sim::PlayerId;

/// Sends a pump cue at a rate from [`PUMP_RATE`] while the boots of the local player jet.
pub fn pump(
    time: Res<Time>,
    players: Query<(&PlayerId, &Gear), With<Player>>,
    mut due: Local<f32>,
    mut cues: MessageWriter<AudioCue>,
) {
    let gear = local(players.iter().map(|(id, gear)| (*id, gear)));
    let Some(gear) = gear.filter(|gear| jetting(&gear.input)) else {
        *due = 0.0;
        return;
    };
    let spool = gear.state.power();
    *due += time.delta_secs() * (PUMP_RATE.0 + (PUMP_RATE.1 - PUMP_RATE.0) * spool);
    if *due >= 1.0 {
        *due %= 1.0;
        cues.write(AudioCue::Pump(spool));
    }
}
