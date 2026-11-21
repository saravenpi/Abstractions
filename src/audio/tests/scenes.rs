//! Scenes of the whole mix for the loudness table: what a player hears in a kind of moment,
//! music, weather, ship and cues together, with the loudness that kind of moment aims at.

use bevy::prelude::*;

use super::planet;
use super::rig::{parked, seated};
use super::takes::{flyby, throttle, warp};
use super::takes_ground::liftoff;
use crate::audio::call::Call;
use crate::audio::cue::{AudioCue, ChirpKind, StepSurface};
use crate::audio::hit::{HitSound, Kind};
use crate::audio::scene::AudioScene;
use crate::audio::ship::feed::{Ch, HullSound};
use crate::audio::spatial::tune::{BUDDY, HIT, STEP, THUNDER};
use crate::audio::spatial::{Between, Listener, Origin, Spot, spot};
use crate::audio::step::Step;
use crate::audio::tune::{BUSY, CALM, TITLE};

/// One scene: its name, its length in seconds, the integrated loudness it aims at in LUFS, the
/// scene at each moment, and the cue of each frame of the game, sixty per second, with how it
/// reaches the listener.
pub(super) type Scene = (
    &'static str,
    f32,
    f32,
    fn(f32) -> AudioScene,
    fn(usize) -> Option<(AudioCue, Spot)>,
);

/// Returns how a sound from `from` reaches a listener standing at the origin, facing ahead, in the
/// air of home.
fn heard(from: Origin) -> Spot {
    spot(&Listener::default(), from, Between::default())
}

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

/// Returns the footsteps of a walk on grass, the listener's own, with the companion saying
/// something now and then from beside the head.
fn walk(frame: usize) -> Option<(AudioCue, Spot)> {
    let feet = Origin::Held(Vec3::NEG_Y * STEP.0, STEP.0, STEP.1);
    let buddy = Origin::At(Vec3::new(0.9, 0.2, -0.6), BUDDY);
    match (frame % 27, frame % 400) {
        (_, 200) => Some((AudioCue::Chirp(ChirpKind::Curious), heard(buddy))),
        (0, _) => Some((
            AudioCue::Footstep(Step {
                left: frame.is_multiple_of(54),
                ..Step::on(StepSurface::Grass)
            }),
            heard(feet),
        )),
        _ => None,
    }
}

/// Returns the thunder of a storm: one near roll, one far, a kilometre off on either side.
fn thunder(frame: usize) -> Option<(AudioCue, Spot)> {
    let off = |side: f32| heard(Origin::At(Vec3::new(side, 0.3, -0.5) * THUNDER, THUNDER));
    match frame {
        240 => Some((AudioCue::Thunder(0.9), off(0.8))),
        620 => Some((AudioCue::Thunder(0.4), off(-0.8))),
        _ => None,
    }
}

/// Returns the cues of a fight beside the listener: a hard wet blow twice a second a metre off
/// to the right, a swing between two blows, and an animal two metres off that calls every other
/// second.
fn fight(frame: usize) -> Option<(AudioCue, Spot)> {
    let blow = |kind: Kind, level: f32| HitSound {
        kind,
        level,
        hz: 420.0,
        hard: 0.6,
        wet: kind == Kind::Clean,
    };
    let hit = heard(Origin::At(Vec3::new(0.3, 0.0, -0.95), HIT.0));
    let animal = heard(Origin::At(Vec3::new(1.0, 0.0, -1.7), 5.6));
    match (frame % 120, frame % 32) {
        (60, _) => Call::unpack(24 | 1 << 9).map(|call| (AudioCue::Call(call), animal)),
        (_, 0) => Some((AudioCue::Hit(blow(Kind::Clean, 1.0)), hit)),
        (_, 16) => Some((AudioCue::Hit(blow(Kind::Whoosh, 0.7)), hit)),
        _ => None,
    }
}

/// Returns the cues of a warp, in the cabin: the drive starts to spool, engages and drops out.
fn jump(frame: usize) -> Option<(AudioCue, Spot)> {
    let cue = match frame {
        60 => AudioCue::WarpSpool,
        300 => AudioCue::WarpEngage,
        600 => AudioCue::WarpExit,
        _ => return None,
    };
    Some((cue, Spot::FLAT))
}

/// Every scene of the table. The seat is 4 dB under the chase view since 0.11.0, at the wish of the
/// owner, so a burn heard from the seat is busy play, no longer the loudest.
pub(super) const SCENES: [Scene; 10] = [
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
        "liftoff_inside",
        13.0,
        BUSY,
        |t| with_ship(liftoff(t), true),
        |_| None,
    ),
    (
        "full_throttle_inside",
        20.0,
        BUSY,
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
