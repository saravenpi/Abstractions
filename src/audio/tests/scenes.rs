//! Scenes of the whole mix for the loudness table: what a player hears in a kind of moment,
//! music, weather, ship and cues together, with the loudness that kind of moment aims at.

use super::calls::calling;
use super::planet;
use super::rig::{parked, seated};
use super::takes::{flyby, throttle, warp};
use super::takes_ground::liftoff;
use crate::audio::call::Call;
use crate::audio::cue::{AudioCue, ChirpKind, StepSurface};
use crate::audio::hit::{HitSound, Kind};
use crate::audio::scene::AudioScene;
use crate::audio::ship::feed::{Ch, HullSound};
use crate::audio::tune::{BUSY, CALM, LOUD, TITLE};
use crate::life::signs::calls::{Cry, Timbre};

/// One scene: its name, its length in seconds, the integrated loudness it aims at in LUFS, the
/// scene at each moment, and the cue of each frame of the game, sixty per second.
pub(super) type Scene = (
    &'static str,
    f32,
    f32,
    fn(f32) -> AudioScene,
    fn(usize) -> Option<AudioCue>,
);

/// Returns a fair day on the ground of a planet, with a light wind and a shore not far off.
fn fair() -> AudioScene {
    AudioScene {
        wind: 0.15,
        shore: 0.25,
        ..planet(5)
    }
}

/// Returns the fair day with one ship in it: from its seat when `aboard`, else from outside.
fn with_ship(hull: HullSound, aboard: bool) -> AudioScene {
    let mut scene = fair();
    scene.hulls[0] = if aboard { seated(hull) } else { hull };
    scene.cabin = f32::from(u8::from(aboard));
    scene
}

/// Returns the scene of a ship in a vacuum, from its seat: no music, no weather.
fn in_space(hull: HullSound, warping: bool) -> AudioScene {
    let mut scene = AudioScene {
        in_space: true,
        altitude_fade: 0.0,
        cabin: 1.0,
        warping,
        ..AudioScene::default()
    };
    scene.hulls[0] = seated(hull.with(Ch::Air, 0.0));
    scene
}

/// Returns the footsteps of a walk on grass, with the companion saying something now and then.
fn walk(frame: usize) -> Option<AudioCue> {
    match (frame % 27, frame % 400) {
        (_, 200) => Some(AudioCue::Chirp(ChirpKind::Curious)),
        (0, _) => Some(AudioCue::Footstep(StepSurface::Grass)),
        _ => None,
    }
}

/// Returns the thunder of a storm: one near roll, one far.
fn thunder(frame: usize) -> Option<AudioCue> {
    match frame {
        240 => Some(AudioCue::Thunder(0.9)),
        620 => Some(AudioCue::Thunder(0.4)),
        _ => None,
    }
}

/// Returns the cues of a fight beside the listener: a hard wet blow twice a second, a swing
/// between two blows, and an animal that calls every other second.
fn fight(frame: usize) -> Option<AudioCue> {
    let blow = |kind: Kind, level: f32| HitSound {
        kind,
        level,
        hz: 420.0,
        hard: 0.6,
        wet: kind == Kind::Clean,
        side: 0.3,
        near: 1.0,
    };
    match (frame % 120, frame % 32) {
        (60, _) => Some(AudioCue::Call(Call {
            side: 0.33,
            near: 0.87,
            metres: 4.0,
            ..calling(Timbre::Low, 320.0, Cry::Contact)
        })),
        (_, 0) => Some(AudioCue::Hit(blow(Kind::Clean, 1.0))),
        (_, 16) => Some(AudioCue::Hit(blow(Kind::Whoosh, 0.7))),
        _ => None,
    }
}

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
/// touch, from 15 to 200 metres away, birds and crickets among them.
pub(super) fn dusk(frame: usize) -> Option<AudioCue> {
    if frame % 60 != 30 {
        return None;
    }
    let k = frame / 60;
    let (timbre, hz) = CHORUS_VOICES[k * 3 % CHORUS_VOICES.len()];
    let metres = 15.0 + (k * 53 % 186) as f32;
    let mut call = calling(timbre, hz, Cry::Contact);
    call.voice.print = (k * 3 % CHORUS_VOICES.len()) as u8 * 37;
    call.one = (k * 11) as u8;
    call.side = ((k * 7 % 9) as f32 / 4.0) - 1.0;
    call.near = 37.0 / (37.0 + metres);
    call.metres = metres;
    Some(AudioCue::Call(call))
}

/// Returns the cues of a warp: the drive starts to spool, engages and drops out.
fn jump(frame: usize) -> Option<AudioCue> {
    match frame {
        60 => Some(AudioCue::WarpSpool),
        300 => Some(AudioCue::WarpEngage),
        600 => Some(AudioCue::WarpExit),
        _ => None,
    }
}

/// Every scene of the table.
pub(super) const SCENES: [Scene; 11] = [
    ("title", 30.0, TITLE, |_| AudioScene::default(), |_| None),
    ("base_calm", 30.0, CALM, |_| fair(), |_| None),
    ("walking", 30.0, CALM, |_| fair(), walk),
    (
        "storm",
        20.0,
        BUSY,
        |_| AudioScene {
            wind: 1.0,
            rain: 1.0,
            ..fair()
        },
        thunder,
    ),
    ("fight", 20.0, BUSY, |_| fair(), fight),
    (
        "dusk_chorus",
        30.0,
        CALM,
        |_| AudioScene {
            daylight: 0.5,
            ..fair()
        },
        dusk,
    ),
    (
        "liftoff_inside",
        13.0,
        LOUD,
        |t| with_ship(liftoff(t), true),
        |_| None,
    ),
    (
        "full_throttle_inside",
        20.0,
        LOUD,
        |_| with_ship(throttle(5.0), true),
        |_| None,
    ),
    (
        "flyby_outside",
        8.0,
        CALM,
        |t| with_ship(flyby(t), false),
        |_| None,
    ),
    (
        "warp_inside",
        13.0,
        BUSY,
        |t| in_space(warp(t), t > 5.0 && t < 10.0),
        jump,
    ),
    (
        "space_cruise_inside",
        20.0,
        CALM,
        |_| in_space(parked().with(Ch::Thrust, 0.3), false),
        |_| None,
    ),
];
