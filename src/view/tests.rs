//! Tests of the camera rig, run against a fake world of boxes.

mod app;
mod blends;
mod cockpit;
mod comfort;
mod cuts;
mod framing;
mod occlusion;
mod rates;
mod ship;
mod steer;
mod world;

use bevy::prelude::*;

use super::basis::frame;
use super::{Kind, Look, Moment, Occluder, Rig, Shown, Target, ViewMode, ViewPrefs};

/// The default field of view setting, 75 degrees, in radians.
const FOV: f32 = 1.308_997;

/// Frame time of the tests that do not care about it.
const DT: f32 = 1.0 / 60.0;

/// Returns the preferences with the third person view on foot and in the ship.
fn third() -> ViewPrefs {
    ViewPrefs {
        on_foot: ViewMode::Third,
        ship: ViewMode::Third,
        ..default()
    }
}

/// Returns a moment for a body of `kind` standing 0.9 m above the origin, seen with `prefs`.
fn moment(kind: Kind, prefs: ViewPrefs) -> Moment {
    Moment {
        target: Target {
            position: Vec3::Y * 0.9,
            kind,
            ..default()
        },
        look: Look::default(),
        prefs,
        fov: FOV,
        dt: DT,
    }
}

/// Steps `rig` for `steps` frames, at least one, while the target moves at its velocity, and
/// returns the last view.
fn run(rig: &mut Rig, moment: &mut Moment, world: &impl Occluder, steps: usize) -> Shown {
    let mut step = || {
        moment.target.position += moment.target.velocity * moment.dt;
        rig.step(moment, world)
    };
    let first = step();
    (1..steps).fold(first, |_, _| step())
}

/// Returns the camera of `shown` relative to `target`, in the axes of the target.
fn seen(shown: &Shown, target: &Target) -> (Vec3, Quat) {
    let back = frame(target).inverse();
    (
        back * (shown.pose.position - target.position),
        back * shown.pose.rotation,
    )
}

/// Returns how far apart two views of a target are: in metres and in radians. The angle
/// comes from the sine of its half, which stays exact for the smallest turns.
fn apart(a: (Vec3, Quat), b: (Vec3, Quat)) -> (f32, f32) {
    let turn = a.1.inverse() * b.1;
    let half = Vec3::new(turn.x, turn.y, turn.z)
        .length()
        .atan2(turn.w.abs());
    (a.0.distance(b.0), 2.0 * half)
}
