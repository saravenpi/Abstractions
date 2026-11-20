//! Tests of wounds and blood: health grows with mass, a wounded animal limps, and blood falls every
//! two metres and fades over the footprint's trail.

use bevy::math::DVec3;

use super::health::Health;
use super::tune::LIMP;
use super::{Blood, Drip};
use crate::life::signs::tune::TRAIL;

/// Health grows with mass; under 40 percent the animal limps, and enough damage kills it.
#[test]
fn wounds_limp_and_kill() {
    let mut small = Health::of(10.0);
    assert!(Health::of(500.0).max > small.max);
    assert_eq!(small.pace(), 1.0);
    assert!(!small.hurt(small.max * 0.7));
    assert_eq!(small.pace(), LIMP);
    assert!(small.hurt(small.max));
}

/// Drops fall two metres apart and fade over the footprint's trail.
#[test]
fn blood_falls_every_two_metres_and_fades() {
    let radius = 1.0e6;
    let mut blood = Blood::default();
    let mut last = None;
    for step in 0..20 {
        let x = 0.51 * f64::from(step);
        let at = (DVec3::Y * radius + DVec3::X * x).normalize();
        let drip = Drip {
            at,
            facing: DVec3::X,
            species: 0,
            size: 0.1,
            time: f64::from(step),
        };
        last = Some(blood.bleed(drip, last, radius));
    }
    assert_eq!(blood.len(), 5);
    let mut signs = Vec::new();
    blood.lay((DVec3::Y, 60.0), (20.0, radius), &mut signs);
    assert_eq!(signs.len(), 5);
    assert!(signs.iter().all(|s| s.fresh > 0.9));
    signs.clear();
    blood.lay((DVec3::Y, 60.0), (TRAIL + 30.0, radius), &mut signs);
    assert!(signs.is_empty() && blood.is_empty());
}
