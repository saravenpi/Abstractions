//! Tests of a javelin meeting the ground: thrown at any angle into a slope its tip is never under
//! the surface in flight, soft ground plants it with the tip buried and the shaft at the angle it
//! came in at, rock turns it away, and one that comes to rest lies along the ground.

use bevy::prelude::*;

use super::super::flight::fly;
use super::aim::cross;
use super::ground::{Ground, Soil};
use super::plant::{Meet, lie, meet, planted};
use super::stage::{Slope, flying};
use super::tune::{DEEPEST, SOILS};

/// Seconds in a frame at sixty frames per second.
const FRAME: f32 = 1.0 / 60.0;

/// Gravity of the tests, in metres per second squared.
const G: Vec3 = Vec3::new(0.0, -9.81, 0.0);

/// Returns the frames of a javelin thrown from `at` at `velocity` over `ground` until it meets it,
/// each tip as the flight leaves it, and the point it met the ground at.
fn flight(ground: &Slope, at: Vec3, velocity: Vec3) -> (Vec<Vec3>, Option<Vec3>) {
    let (mut tip, mut v, mut tips) = (at, velocity, vec![at]);
    for _ in 0..600 {
        let (to, next) = fly(tip, v, G, 0.0, FRAME);
        if let Some(met) = cross(ground, tip, to) {
            tips.push(met);
            return (tips, Some(met));
        }
        (tip, v) = (to, next);
        tips.push(tip);
    }
    (tips, None)
}

/// Thrown up, level and down, uphill, downhill and across slopes from flat to steep, the tip is on
/// or over the ground on every frame, and every throw comes down on it.
#[test]
fn a_throw_never_goes_under_a_slope() {
    let soft = Soil::Soft(SOILS[0]);
    for slope in [0.0_f32, 0.35, 0.7] {
        for toward in [Vec3::NEG_Z, Vec3::Z, Vec3::X] {
            let ground = Slope::rising(slope, toward, soft);
            for pitch in [-1.0_f32, -0.5, 0.0, 0.5, 1.0] {
                let start = ground.lift(Vec3::ZERO) + Vec3::Y * 1.4;
                let velocity = Vec3::new(0.0, pitch.sin(), -pitch.cos()) * 24.0;
                let (tips, met) = flight(&ground, start, velocity);
                let low = tips
                    .iter()
                    .map(|t| ground.clearance(*t))
                    .fold(f32::MAX, f32::min);
                assert!(low >= -1e-4, "slope {slope} {toward}, pitch {pitch}: {low}");
                assert!(
                    met.is_some(),
                    "slope {slope} {toward}, pitch {pitch} never landed"
                );
            }
        }
    }
}

/// On soft earth a fast javelin that leads plants: its tip is buried no deeper than the most, and
/// its shaft stands at the angle it came in at, its butt out of the ground. On rock it glances
/// off, and so does one that arrives too flat.
#[test]
fn soft_ground_plants_and_rock_glances() {
    let way = Vec3::new(0.0, -0.6, -0.8);
    let javelin = flying(Vec3::ZERO, way * 22.0);
    let ground = Slope::rising(0.0, Vec3::NEG_Z, Soil::Soft(SOILS[0]));
    let Meet::Plant(deep) = meet(&javelin, way, (Vec3::Y, ground.soil)) else {
        panic!("it did not plant");
    };
    assert!(deep > 0.0 && deep <= DEEPEST, "{deep}");
    let stands = planted(Vec3::ZERO, way, deep);
    assert!(ground.clearance(stands.translation) < -0.01, "{stands:?}");
    assert!((stands.rotation * Vec3::Y).angle_between(way) < 1e-3);
    let butt = stands.translation - stands.rotation * Vec3::Y * 1.3;
    assert!(ground.clearance(butt) > 0.5, "{butt}");
    let Meet::Glance(off) = meet(&javelin, way, (Vec3::Y, Soil::Hard)) else {
        panic!("it planted in rock");
    };
    assert!(off.y > 0.0, "{off}");
    let flat = Vec3::new(0.0, -0.1, -1.0).normalize();
    let skims = flying(Vec3::ZERO, flat * 22.0);
    assert!(matches!(
        meet(&skims, flat, (Vec3::Y, ground.soil)),
        Meet::Glance(_)
    ));
}

/// A javelin that came to rest on a slope lies along it, the tip and the butt on the ground and
/// nowhere under it.
#[test]
fn a_resting_javelin_lies_on_the_slope() {
    let ground = Slope::rising(0.4, Vec3::X, Soil::Hard);
    let tip = ground.lift(Vec3::new(1.0, 0.0, -2.0));
    let place = lie(&ground, tip, Vec3::new(0.3, -0.9, -0.3).normalize(), 1.3);
    let butt = place.translation - place.rotation * Vec3::Y * 1.3;
    for end in [place.translation, butt] {
        let clear = ground.clearance(end);
        assert!((0.0..0.05).contains(&clear), "{end}: {clear}");
    }
}
