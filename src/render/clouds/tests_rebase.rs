//! Tests that the clouds are laid in axes fixed to the planet: moving the render origin or turning
//! the render axes, as a rebase of the floating origin does, leaves every point of the field where
//! it was, so the drift carries on from the same place.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use crate::render::cloudshade::fixed;
use crate::render::{Observer, Vantage};
use crate::weather::{CELL, density_at};

/// Returns the planet-fixed unit direction the shells read for the point `point` of the star
/// system, seen by `observer` with the camera at the render position `camera`: the direction
/// from the centre of the shell to that point in render space, turned by the rotation the
/// shaders are handed.
fn seen(observer: Observer, camera: Vec3, point: DVec3) -> Vec3 {
    let vantage = Vantage {
        observer,
        camera,
        ..default()
    };
    let centre = observer.place(observer.center, camera);
    let at = observer.place(point, camera);
    fixed(&vantage) * (at - centre).normalize()
}

/// A rebase moves the camera in render space and may turn the render axes; the cloud over a place
/// of the ground reads the same spot of the noise before and after it, at the same second.
#[test]
fn a_rebase_leaves_the_clouds_in_place() {
    let spin = DQuat::from_rotation_y(0.7);
    let center = DVec3::new(1.5e11, 2.0e9, -4.0e10);
    let eye = center + spin * DVec3::new(0.0, 6_372_000.0, 0.0);
    let before = Observer {
        eye,
        center,
        spin,
        to_render: DQuat::from_rotation_x(0.2),
    };
    let after = Observer {
        to_render: DQuat::from_rotation_x(0.2) * DQuat::from_rotation_z(0.01),
        ..before
    };
    let cells = 6_376_000.0 / CELL;
    for offset in [
        DVec3::new(3000.0, 4000.0, -1000.0),
        DVec3::new(-20_000.0, 4500.0, 7000.0),
    ] {
        let point = eye + spin * offset;
        let a = seen(before, Vec3::ZERO, point);
        let b = seen(after, Vec3::new(512.0, -30.0, 900.0), point);
        assert!(a.distance(b) < 2.0e-6, "{a} then {b}");
        for seconds in [10.0, 86_400.0, 3.6e7] {
            let cloud = |dir: Vec3| density_at(dir.as_dvec3(), cells, seconds, 0.6);
            assert!((cloud(a) - cloud(b)).abs() < 1.0e-3, "{seconds}");
        }
    }
}
