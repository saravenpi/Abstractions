//! Tests of how damage shows: the marks, the screen of the head and the pose of a hurt body.

use bevy::prelude::*;

use super::super::pose::{Core, Limb, Pose};
use super::eye::dim;
use super::limp::limp;
use super::plates;
use super::scar::{count, slab};
use super::tune::{DIM, HANG, LOST, MARK_THIN, SCARS};
use crate::condition::{Condition, Part};

/// A whole part carries no mark, a destroyed one the most, and the number only grows with the harm.
#[test]
fn marks_grow_with_harm() {
    assert_eq!(count(1.0), 0);
    assert_eq!(count(0.0), SCARS);
    assert!((1..=SCARS).all(|n| count(1.0 - n as f32 / SCARS as f32 + 0.01) <= n));
    assert!(count(0.7) < count(0.3));
}

/// Every mark lies on a face of a plate of its part, a hair above it.
#[test]
fn marks_lie_on_their_plate() {
    for part in Part::ALL {
        for plate in plates::of(part) {
            for slot in 0..SCARS {
                let place = slab(plate, part, slot);
                assert!(plate.distance(place.translation) <= MARK_THIN);
            }
        }
    }
}

/// A whole head keeps the screen at full light; a hurt one is dimmer, and the worst drops out
/// now and then.
#[test]
fn a_hurt_head_dims_and_drops_out() {
    assert!((0..100).all(|t| dim(1.0, t as f32 * 0.037) == 1.0));
    let lights: Vec<f32> = (0..400).map(|t| dim(0.2, t as f32 * 0.037)).collect();
    assert!(
        lights
            .iter()
            .all(|light| *light <= 1.0 - 0.8 * DIM + 1e-4 || *light == LOST)
    );
    assert!(lights.contains(&LOST));
    assert!(lights.iter().any(|light| *light > 0.4));
}

/// A hurt leg swings less and the body rolls toward the sound one, and a broken arm hangs.
#[test]
fn a_hurt_body_limps_and_hangs() {
    let walking = Pose::default().both(Limb::Hip, 0.5).both(Limb::Knee, 0.6);
    let mut body = Condition::default();
    body.hurt(Part::LegR, 1.0e6, true);
    body.hurt(Part::ArmL, 1.0e6, true);
    let limped = limp(walking, &body, 5.0);
    assert!(limped.limb(Limb::Hip, 1) < limped.limb(Limb::Hip, 0));
    assert!(limped.core(Core::Roll) > 0.0);
    assert_eq!(limped.limb(Limb::Shoulder, 0), HANG[0]);
    assert_eq!(limp(walking, &Condition::default(), 5.0), walking);
}

/// Two lost legs lean the body forward into a crawl.
#[test]
fn lost_legs_crawl() {
    let mut body = Condition::default();
    body.hurt(Part::LegL, 1.0e6, true);
    body.hurt(Part::LegR, 1.0e6, true);
    assert!(limp(Pose::default(), &body, 1.0).core(Core::Pitch) > 0.3);
}
