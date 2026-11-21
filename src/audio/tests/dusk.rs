//! The chorus at dusk: once a second an animal of one of seven species keeps in touch from somewhere
//! around the listener, for the loudness table and the takes of the animals.

use bevy::prelude::*;

use super::calls::calling;
use crate::audio::cue::AudioCue;
use crate::audio::spatial::{Between, Listener, Origin, Spot, spot};
use crate::life::signs::calls::{Cry, Timbre};

/// The timbres and pitches of the seven species of the chorus at dusk.
const CHORUS_VOICES: [(Timbre, f32); 7] = [
    (Timbre::Whistle, 2400.0),
    (Timbre::Whistle, 1700.0),
    (Timbre::Low, 140.0),
    (Timbre::Hoot, 320.0),
    (Timbre::Click, 4000.0),
    (Timbre::Whistle, 3100.0),
    (Timbre::Croak, 260.0),
];

/// Returns the calls of a chorus at dusk: once a second an animal of one of seven species keeps in
/// touch, from 15 to 200 metres away all round, birds and crickets among them.
pub(super) fn dusk(frame: usize) -> Option<(AudioCue, Spot)> {
    if frame % 60 != 30 {
        return None;
    }
    let k = frame / 60;
    let (timbre, hz) = CHORUS_VOICES[k * 3 % CHORUS_VOICES.len()];
    let metres = 15.0 + (k * 53 % 186) as f32;
    let mut call = calling(timbre, hz, Cry::Contact);
    call.voice.print = (k * 3 % CHORUS_VOICES.len()) as u8 * 37;
    call.one = (k * 11) as u8;
    let side = ((k * 7 % 9) as f32 / 4.0) - 1.0;
    let at = Vec3::new(side, 0.0, -(1.0 - side * side).sqrt()) * metres;
    Some((
        AudioCue::Call(call),
        spot(
            &Listener::default(),
            Origin::At(at, 3.7),
            Between::default(),
        ),
    ))
}
