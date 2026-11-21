//! The offline renderer of the animals: named takes of calls played through the real mixer on a
//! fair day with the music off, written as WAV files beside the takes of the ship.
//!
//! It runs only when `SHIP_AUDIO_DIR` names the folder to write to:
//! `SHIP_AUDIO_DIR=/tmp/ship-audio cargo test --features dev --lib -- audio::tests::render --nocapture`.
//!
//! Each call is built as the hearing builds it: the size of the animal follows from its pitch by the
//! frequency law of `crate::life::signs::voice`, its tract from its size, and its level from its
//! distance and size by the law of `crate::fauna::signs`.

use std::path::Path;

use super::render::write;
use super::scenes::dusk;
use super::{frames, mixer, planet};
use crate::audio::call::Call;
use crate::audio::cue::AudioCue;
use crate::audio::dsp::limit::CEILING;
use crate::audio::scene::AudioScene;
use crate::life::signs::calls::{Cry, Timbre, Voice};
use crate::settings::Audio as Volumes;

/// One call of a take: when, timbre, pitch in hertz, cry, metres away, side, voice print.
type Row = (f32, Timbre, f32, Cry, f32, f32, u8);

/// One take: file name, seconds, daylight, its calls, and whether the chorus at dusk sounds in it.
type Take = (&'static str, f32, f32, &'static [Row], bool);

/// A herd of big grazers keeping in touch 30 metres off, a young answering.
const HERD: [Row; 5] = [
    (0.5, Timbre::Low, 105.0, Cry::Contact, 28.0, -0.4, 0x21),
    (2.6, Timbre::Low, 98.0, Cry::Contact, 33.0, 0.3, 0x21),
    (4.4, Timbre::Low, 104.0, Cry::Young, 30.0, 0.1, 0x21),
    (5.8, Timbre::Low, 102.0, Cry::Contact, 31.0, 0.6, 0x21),
    (7.6, Timbre::Low, 96.0, Cry::Contact, 36.0, -0.1, 0x21),
];

/// A herd that has seen the robot: big ones blow, then all call the alarm as they run off.
const ALARM: [Row; 5] = [
    (0.4, Timbre::Low, 150.0, Cry::Alarm, 14.0, 0.2, 0x46),
    (1.3, Timbre::Hoot, 200.0, Cry::Alarm, 18.0, -0.3, 0x47),
    (2.4, Timbre::Low, 155.0, Cry::Alarm, 24.0, 0.5, 0x46),
    (3.7, Timbre::Hoot, 194.0, Cry::Alarm, 32.0, 0.1, 0x47),
    (5.1, Timbre::Low, 148.0, Cry::Alarm, 48.0, -0.2, 0x46),
];

/// A hunter that snorts and growls, its growl winding up rough, then charges crying.
const PREDATOR: [Row; 3] = [
    (0.5, Timbre::Growl, 147.0, Cry::Threat, 14.0, 0.1, 0x93),
    (4.2, Timbre::Growl, 150.0, Cry::Chase, 9.0, 0.0, 0x93),
    (6.6, Timbre::Growl, 145.0, Cry::Chase, 6.0, -0.2, 0x93),
];

/// A wounded grazer crying in distress.
const HURT: [Row; 3] = [
    (0.5, Timbre::Low, 173.0, Cry::Distress, 6.0, 0.3, 0x58),
    (2.9, Timbre::Low, 176.0, Cry::Distress, 6.0, 0.3, 0x58),
    (5.4, Timbre::Low, 170.0, Cry::Distress, 6.0, 0.3, 0x58),
];

/// A grazer crying as it is hurt, then dying.
const DYING: [Row; 3] = [
    (0.5, Timbre::Low, 173.0, Cry::Distress, 5.0, -0.2, 0x58),
    (2.6, Timbre::Low, 176.0, Cry::Distress, 5.0, -0.2, 0x58),
    (4.8, Timbre::Low, 170.0, Cry::Death, 5.0, -0.2, 0x58),
];

/// Small animals near the head: a cricket chirping, then a tiny bird.
const SMALL: [Row; 7] = [
    (0.3, Timbre::Click, 4000.0, Cry::Contact, 3.0, 0.4, 0x01),
    (0.9, Timbre::Click, 4000.0, Cry::Contact, 3.0, 0.4, 0x01),
    (1.5, Timbre::Click, 4000.0, Cry::Contact, 3.0, 0.4, 0x01),
    (2.1, Timbre::Click, 4000.0, Cry::Contact, 3.0, 0.4, 0x01),
    (3.2, Timbre::Rasp, 1800.0, Cry::Contact, 4.0, -0.5, 0x02),
    (4.4, Timbre::Whistle, 3500.0, Cry::Contact, 6.0, -0.3, 0x00),
    (5.6, Timbre::Whistle, 3600.0, Cry::Contact, 6.0, -0.3, 0x00),
];

/// A huge animal, two tonnes and more, calling from 40 metres.
const HUGE: [Row; 2] = [
    (0.5, Timbre::Low, 45.0, Cry::Contact, 40.0, 0.2, 0xA9),
    (5.2, Timbre::Low, 47.0, Cry::Contact, 40.0, 0.2, 0xA9),
];

/// Birds of five species singing at dawn, near and far, each its own melody.
const DAWN: [Row; 8] = [
    (0.3, Timbre::Whistle, 2600.0, Cry::Contact, 12.0, -0.6, 0x00),
    (1.2, Timbre::Whistle, 3400.0, Cry::Contact, 30.0, 0.5, 0x14),
    (2.0, Timbre::Whistle, 2100.0, Cry::Contact, 8.0, 0.1, 0x28),
    (3.1, Timbre::Whistle, 2700.0, Cry::Contact, 14.0, -0.5, 0x00),
    (3.9, Timbre::Whistle, 3900.0, Cry::Contact, 60.0, 0.8, 0x3C),
    (4.8, Timbre::Whistle, 2900.0, Cry::Contact, 22.0, -0.2, 0x24),
    (6.0, Timbre::Whistle, 3300.0, Cry::Contact, 28.0, 0.4, 0x14),
    (7.1, Timbre::Whistle, 2200.0, Cry::Contact, 9.0, 0.2, 0x28),
];

/// Every take, in the order the files are numbered after those of the ship.
const TAKES: [Take; 9] = [
    ("21_herd_grazing_30m", 9.0, 1.0, &HERD, false),
    ("22_alarm", 7.0, 1.0, &ALARM, false),
    ("23_predator_growl_charge", 9.0, 1.0, &PREDATOR, false),
    ("24_hurt", 8.0, 1.0, &HURT, false),
    ("25_dying", 9.0, 1.0, &DYING, false),
    ("26_small_animals", 7.0, 1.0, &SMALL, false),
    ("27_huge_animal", 10.0, 1.0, &HUGE, false),
    ("28_birds_dawn", 9.0, 0.5, &DAWN, false),
    ("29_chorus_dusk", 10.0, 0.5, &[], true),
];

/// Frames between two frames of the game.
const GAME_FRAME: usize = 800;

/// Returns the call of `row`: the size of the animal from its pitch, its tract from its size, its level
/// from its size and distance.
fn call(&(at, timbre, hz, cry, metres, side, print): &Row) -> Call {
    let kg = (1000.0 / hz).powf(2.5);
    let length = 2.5 * (kg / 1000.0).cbrt();
    let half = 12.0 * kg.cbrt();
    let voice = Voice {
        hz,
        timbre,
        notes: 2,
        muffled: false,
        formant: (350.0 / (0.3 * length)).clamp(60.0, 8000.0),
        print,
    };
    Call {
        voice,
        cry,
        one: (at * 37.0) as u8,
        side,
        near: half / (half + metres),
        metres,
    }
}

/// Returns the cues of a take with the frame each one starts on: its calls, and the chorus at dusk
/// once per game frame when it has one.
fn cues((_, seconds, _, rows, chorus): &Take) -> Vec<(usize, AudioCue)> {
    let mut cues: Vec<(usize, AudioCue)> = rows
        .iter()
        .map(|row| (frames(row.0), AudioCue::Call(call(row))))
        .collect();
    let games = if *chorus {
        frames(*seconds) / GAME_FRAME
    } else {
        0
    };
    cues.extend((0..games).filter_map(|g| dusk(g).map(|cue| (g * GAME_FRAME, cue))));
    cues
}

/// Renders a take through the mixer, music off and a light wind, the scene once per game frame.
fn record(take: &Take) -> Vec<[f32; 2]> {
    let scene = AudioScene {
        daylight: take.2,
        wind: 0.1,
        ..planet(5)
    };
    let volumes = Volumes {
        music: 0.0,
        ..Volumes::default()
    };
    let cues = cues(take);
    let (mut mixer, shared) = mixer(&scene);
    let mut out = Vec::with_capacity(frames(take.1));
    for i in 0..frames(take.1) {
        if i % GAME_FRAME == 0 {
            shared.publish(&scene, &volumes);
        }
        for (_, cue) in cues.iter().filter(|(at, _)| *at == i) {
            shared.push(*cue);
        }
        out.push(mixer.frame());
    }
    out
}

/// Renders every take of the animals into the folder `SHIP_AUDIO_DIR` names, and does nothing without
/// it. Every take stays within full scale and is heard.
#[test]
fn fauna_takes_render() {
    let Some(dir) = std::env::var_os("SHIP_AUDIO_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).expect("the folder can be made");
    for take in &TAKES {
        let sound = record(take);
        let peak = sound.iter().flatten().fold(0.0_f32, |p, x| p.max(x.abs()));
        assert!(peak <= CEILING && peak > 0.01, "{}: {peak}", take.0);
        let path = Path::new(&dir).join(format!("{}.wav", take.0));
        write(&path, &sound).expect("the file can be written");
        println!("{}: peak {peak:.3}", path.display());
    }
}
