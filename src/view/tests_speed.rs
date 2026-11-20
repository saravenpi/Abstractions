//! Tests of the widening of the view with speed and of the "Speed effects" setting.

use bevy::prelude::*;

use super::tune::MAX_FOV;
use super::{Kind, Look, Moment, Rig, Target, ViewPrefs};

/// The field of view setting, 75 degrees in radians.
const FOV: f32 = 1.308_997;

/// Frame time in seconds.
const DT: f32 = 1.0 / 60.0;

/// Returns the field of view the rig settles on, after six seconds of a body of `kind` flying at
/// `speed` metres per second, with the "Speed effects" setting at `strength`.
fn settled(kind: Kind, speed: f32, strength: f32) -> f32 {
    let mut rig = Rig::default();
    let mut moment = Moment {
        target: Target {
            position: Vec3::Y * 500.0,
            velocity: Vec3::NEG_Z * speed,
            kind,
            ..default()
        },
        look: Look::default(),
        prefs: ViewPrefs {
            speed_fx: strength,
            ..default()
        },
        fov: FOV,
        dt: DT,
    };
    let open = |_: Vec3, _: Vec3, _: f32, _: f32| None;
    let mut fov = FOV;
    for _ in 0..360 {
        moment.target.position += moment.target.velocity * DT;
        fov = rig.step(&moment, &open).pose.fov;
    }
    fov
}

/// At rest the view has the field of view that was set, in every view.
#[test]
fn the_view_is_not_widened_at_rest() {
    for kind in [Kind::OnFoot, Kind::Flying, Kind::Ship] {
        let fov = settled(kind, 0.0, 1.0);
        assert!((fov - FOV).abs() < 1.0e-4, "{kind:?}: {fov}");
    }
}

/// The widening grows with speed, never shrinks and never passes the widest view, up to a warp.
#[test]
fn the_widening_grows_with_speed_and_is_capped() {
    for kind in [Kind::Flying, Kind::Ship] {
        let speeds = [0.0, 10.0, 30.0, 60.0, 200.0, 5.0e3, 1.0e5, 1.0e8];
        let fovs = speeds.map(|speed| settled(kind, speed, 1.0));
        assert!(
            fovs.windows(2).all(|pair| pair[0] <= pair[1] + 1.0e-5),
            "{fovs:?}"
        );
        assert!(fovs.iter().all(|fov| *fov <= MAX_FOV + 1.0e-5), "{fovs:?}");
        assert!(
            fovs[3] > FOV + 0.01 && fovs[3] - FOV < 0.35,
            "{kind:?}: {fovs:?}"
        );
    }
}

/// With the setting at 0 the field of view stays the one that was set, at any speed.
#[test]
fn the_setting_at_zero_turns_the_widening_off() {
    for kind in [Kind::OnFoot, Kind::Flying, Kind::Ship] {
        for speed in [30.0, 120.0, 1.0e5] {
            let fov = settled(kind, speed, 0.0);
            assert!((fov - FOV).abs() < 1.0e-4, "{kind:?} at {speed}: {fov}");
        }
    }
}

/// Half the setting gives a widening between none and the full one.
#[test]
fn the_setting_scales_the_widening() {
    let (none, half, full) = (
        settled(Kind::Flying, 80.0, 0.0),
        settled(Kind::Flying, 80.0, 0.5),
        settled(Kind::Flying, 80.0, 1.0),
    );
    assert!(none < half && half < full);
}
