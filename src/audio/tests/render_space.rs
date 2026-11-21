//! Takes of the spatial path for the owner: an animal calling as it walks round the listener, the
//! same call near and far, in thin air and on a dry desert world, a blow on either side, and the
//! companion floating round the head. Every take goes through the real mixer and the real spots.
//!
//! It runs only when `SHIP_AUDIO_DIR` names the folder to write to:
//! `SHIP_AUDIO_DIR=/tmp/ship-audio cargo test --features dev --lib -- audio::tests::render --nocapture`.

use std::f32::consts::TAU;
use std::path::Path;

use bevy::prelude::*;

use super::render::write;
use super::rig::alone;
use super::{RATE, frames, mixer};
use crate::audio::call::Call;
use crate::audio::cue::{AudioCue, ChirpKind};
use crate::audio::dsp::limit::CEILING;
use crate::audio::hit::{HitSound, Kind};
use crate::audio::ship::feed::HullSound;
use crate::audio::spatial::tune::{BUDDY, CALL, HIT};
use crate::audio::spatial::{Air, Between, Listener, Origin, Spot, spot};
use crate::life::signs::calls::{Cry, Timbre, Voice};
use crate::universe::PlanetRecord;

/// One sound of a take: the second it starts at, what it is, and how it reaches the ears.
type Moment = (f32, AudioCue, Spot);

/// Reach of the call of an animal of 200 kilograms, as the hearing of the fauna gives it.
const REACH: f32 = CALL * 12.0 * 5.85;

/// The call every take of an animal plays: a low contact call of three notes.
const MOO: Call = Call {
    voice: Voice {
        hz: 180.0,
        timbre: Timbre::Low,
        notes: 3,
        muffled: false,
    },
    cry: Cry::Contact,
};

/// Returns how a sound at `at` of `reach` reaches a listener at the origin facing ahead in `air`.
fn heard(air: Air, at: Vec3, reach: f32) -> Spot {
    let ear = Listener {
        air,
        ..Listener::default()
    };
    spot(&ear, Origin::At(at, reach), Between::default())
}

/// Returns the call every second of an animal walking once round the listener at 20 metres in
/// twelve seconds, starting ahead and going right.
fn circle(air: Air) -> Vec<Moment> {
    (0..12)
        .map(|n| {
            let turn = n as f32 / 12.0 * TAU;
            let at = Vec3::new(turn.sin(), 0.0, -turn.cos()) * 20.0;
            (0.5 + n as f32, AudioCue::Call(MOO), heard(air, at, REACH))
        })
        .collect()
}

/// Returns two calls of an animal `metres` away, 30 degrees to the right, in `air`.
fn calls(metres: f32, air: Air) -> Vec<Moment> {
    let at = Vec3::new(0.5, 0.0, -0.866) * metres;
    [0.6, 2.4]
        .map(|t| (t, AudioCue::Call(MOO), heard(air, at, REACH)))
        .to_vec()
}

/// Returns four hard blows a metre and a half to the `side`, -1 for the left, 1 for the right.
fn blows(side: f32) -> Vec<Moment> {
    let blow = HitSound {
        kind: Kind::Clean,
        level: 0.8,
        hz: 700.0,
        hard: 0.7,
        wet: false,
    };
    let at = Vec3::new(1.5 * side, 0.0, -0.3);
    [0.5, 1.1, 1.7, 2.3]
        .map(|t| (t, AudioCue::Hit(blow), heard(Air::home(), at, HIT.0)))
        .to_vec()
}

/// Returns the companion chirping as it floats twice round the head at 1.2 metres.
fn buddy() -> Vec<Moment> {
    let kinds = [
        ChirpKind::Happy,
        ChirpKind::Curious,
        ChirpKind::Delight,
        ChirpKind::Hum,
    ];
    (0..12)
        .map(|n| {
            let turn = n as f32 / 6.0 * TAU;
            let at = Vec3::new(turn.sin(), 0.3, -turn.cos()) * 1.2;
            let cue = AudioCue::Chirp(kinds[n % kinds.len()]);
            (0.5 + 0.8 * n as f32, cue, heard(Air::home(), at, BUDDY))
        })
        .collect()
}

/// Returns every take: its file name, its length in seconds and its sounds.
fn takes() -> Vec<(&'static str, f32, Vec<Moment>)> {
    let home = Air::home();
    let thin = Air::sea(&PlanetRecord {
        pressure: 0.1,
        ..PlanetRecord::TEMPERATE
    });
    let desert = Air::sea(&crate::wind::samples()[2]);
    vec![
        ("30_call_circling_20m", 13.0, circle(home)),
        ("31_call_10m", 5.0, calls(10.0, home)),
        ("32_call_50m", 5.0, calls(50.0, home)),
        ("33_call_200m", 5.0, calls(200.0, home)),
        ("34_call_50m_sea_level", 5.0, calls(50.0, home)),
        ("35_call_50m_thin_air", 5.0, calls(50.0, thin)),
        ("36_call_50m_dry_desert", 5.0, calls(50.0, desert)),
        ("37_hit_left", 3.5, blows(-1.0)),
        ("38_hit_right", 3.5, blows(1.0)),
        ("39_companion_circling", 11.0, buddy()),
    ]
}

/// Renders `seconds` of a still, silent day with `moments` played as they fall due.
fn record(seconds: f32, moments: &[Moment]) -> Vec<[f32; 2]> {
    let (mut mixer, shared) = mixer(&alone(HullSound::default()));
    let mut due = moments.iter().peekable();
    (0..frames(seconds))
        .map(|i| {
            let t = i as f32 / RATE;
            while let Some((_, cue, spot)) = due.next_if(|(at, ..)| *at <= t) {
                shared.push(*cue, spot);
            }
            mixer.frame()
        })
        .collect()
}

/// Renders every take of the spatial path into the folder `SHIP_AUDIO_DIR` names, and does nothing
/// without it. Every take stays within full scale and is heard.
#[test]
fn space_takes_render() {
    let Some(dir) = std::env::var_os("SHIP_AUDIO_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).expect("the folder can be made");
    for (name, seconds, moments) in takes() {
        let sound = record(seconds, &moments);
        let peak = sound.iter().flatten().fold(0.0_f32, |p, x| p.max(x.abs()));
        assert!(peak <= CEILING && peak > 1.0e-4, "{name}: {peak}");
        let path = Path::new(&dir).join(format!("{name}.wav"));
        write(&path, &sound).expect("the file can be written");
        println!("{}: peak {peak:.4}", path.display());
    }
}
