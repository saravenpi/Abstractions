//! Takes of the footsteps for the offline renderer: a body walks, runs, creeps, jumps, slides or
//! passes by, its real leg cycle turns that into steps, and each step is played on a ground and
//! from a place. `render_steps` plays them over a calm day and writes them down.

use bevy::prelude::*;

use crate::audio::cue::StepSurface;
use crate::audio::spatial::tune::STEP;
use crate::audio::spatial::{Air, Between, Listener, Origin, Spot, spot};
use crate::audio::step::Step;
use crate::avatar::tread::stage::{live, moving};
use crate::avatar::{AvatarPose, Mode};
use crate::universe::STANDARD_GRAVITY;

/// Gravity of home, in metres per second squared.
const HOME: f32 = STANDARD_GRAVITY as f32;

/// One take: its file name, its length in seconds, its ground, what the body does, and where
/// it is for the listener at each second, as a distance and a side, 0 metres for the listener's
/// own body.
pub(super) type Take = (
    String,
    f32,
    StepSurface,
    fn(f32) -> AvatarPose,
    fn(f32) -> (f32, f32),
);

/// Returns a walk at the walking speed of the game.
pub(super) fn walk(_: f32) -> AvatarPose {
    moving(Mode::Walk, 5.0, 0.0)
}

/// Returns a walk with a jump at 2 seconds that comes down at 4.4 metres per second, the fall
/// from a metre, and a jump off a ledge at 5 seconds that comes down at 8 metres per second.
fn jumps(t: f32) -> AvatarPose {
    let fall = |speed: f32| AvatarPose {
        vertical_speed: -speed,
        ..moving(Mode::Fall, 5.0, 0.0)
    };
    match t {
        _ if (2.0..2.45).contains(&t) => fall(4.4),
        _ if (5.0..5.8).contains(&t) => fall(8.0),
        _ => walk(t),
    }
}

/// Returns a glide that touches down at 1.5 seconds and slides for a second as it slows.
pub(super) fn slide(t: f32) -> AvatarPose {
    match t {
        _ if t < 1.5 => AvatarPose {
            vertical_speed: -3.0,
            ..moving(Mode::Glide, 14.0, 0.0)
        },
        _ if t < 2.5 => moving(Mode::Skid, 14.0 - 10.0 * (t - 1.5), 0.0),
        _ => walk(t),
    }
}

/// Returns where a friend is who walks past along a line 10 metres ahead, from the left to the
/// right, abeam at 3 seconds.
fn past(t: f32) -> (f32, f32) {
    let along = 5.0 * (t - 3.0);
    let far = along.hypot(10.0);
    (far, along / far)
}

/// Where a body is for the listener at each second: a distance and a side.
pub(super) type Place = fn(f32) -> (f32, f32);

/// A take of the gait: its name, its length in seconds, the ground, the pose at each second and
/// where the listener stands.
type Moment = (&'static str, f32, StepSurface, fn(f32) -> AvatarPose, Place);

/// The moments of the gait beyond a walk and a run: two kinds of landing, a crouched walk on
/// soft and on loose ground, a slide after a glide, a friend who walks past, and a walk with no
/// air.
const MOMENTS: [Moment; 7] = [
    (
        "s23_jump_landings_grass",
        7.0,
        StepSurface::Grass,
        jumps,
        |_| (0.0, 0.0),
    ),
    (
        "s24_jump_landings_metal",
        7.0,
        StepSurface::Metal,
        jumps,
        |_| (0.0, 0.0),
    ),
    (
        "s25_crouch_walk_soil",
        6.0,
        StepSurface::Soil,
        |_| moving(Mode::Walk, 2.25, 1.0),
        |_| (0.0, 0.0),
    ),
    (
        "s26_crouch_walk_gravel",
        6.0,
        StepSurface::Gravel,
        |_| moving(Mode::Walk, 2.25, 1.0),
        |_| (0.0, 0.0),
    ),
    ("s27_glide_skid_soil", 5.0, StepSurface::Soil, slide, |_| {
        (0.0, 0.0)
    }),
    (
        "s28_friend_passes_10m_gravel",
        6.0,
        StepSurface::Gravel,
        walk,
        past,
    ),
    ("s29_vacuum_walk_rock", 5.0, StepSurface::Rock, walk, |_| {
        (0.0, 0.0)
    }),
];

/// Every take: a walk and a sprint on every ground, then the moments of the gait.
pub(super) fn takes() -> Vec<Take> {
    let own: Place = |_| (0.0, 0.0);
    let sprint: fn(f32) -> AvatarPose = |_| moving(Mode::Run, 8.5, 0.0);
    let grounds = StepSurface::ALL
        .into_iter()
        .enumerate()
        .flat_map(|(i, surface)| {
            let name = format!("{surface:?}").to_lowercase();
            [
                (
                    format!("s{:02}_walk_{name}", 2 * i + 1),
                    5.0,
                    surface,
                    walk as fn(f32) -> AvatarPose,
                    own,
                ),
                (
                    format!("s{:02}_run_{name}", 2 * i + 2),
                    5.0,
                    surface,
                    sprint,
                    own,
                ),
            ]
        });
    let moments = MOMENTS.map(|(name, seconds, surface, script, place)| {
        (name.to_string(), seconds, surface, script, place)
    });
    grounds.chain(moments).collect()
}

/// Returns how a step `far` metres off on `side` reaches a listener in `air`: the listener's own
/// at no distance, from under the ears.
pub(super) fn heard(far: f32, side: f32, air: Air) -> Spot {
    let ear = Listener {
        air,
        ..Listener::default()
    };
    let ahead = (1.0 - side * side).max(0.0).sqrt();
    let from = match far > 0.0 {
        true => Origin::At(
            Vec3::new(side, 0.0, -ahead) * far - Vec3::Y * STEP.0,
            STEP.0,
        ),
        false => Origin::Held(Vec3::NEG_Y * STEP.0, STEP.0, STEP.1),
    };
    spot(&ear, from, Between::default())
}

/// Returns the steps of `take` with the second each is due at, on its ground, and how each reaches
/// a listener in `air`.
pub(super) fn steps(
    (_, seconds, surface, script, place): &Take,
    air: Air,
) -> Vec<(f32, Step, Spot)> {
    let placed = |(t, step): (f32, Step)| {
        let (far, side) = place(t);
        let step = Step {
            surface: *surface,
            depth: 0.15,
            ..step
        };
        (t, step, heard(far, side, air))
    };
    live(script, HOME, *seconds)
        .into_iter()
        .map(placed)
        .collect()
}
