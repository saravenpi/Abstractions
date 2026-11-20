//! Gear tests: the legs hang from their hinge in the part that carries them, folded and down.

use bevy::prelude::*;

use super::frame::fuselage;
use super::struts::{MAIN, NOSE, leg, pod};
use super::tests_fit::{corners, inside};
use super::wing::wing;
use super::{GEAR, hinges};
use crate::ship::gear::FOLDED;

/// Hangs a leg from `hinge` with its length scaled by `share`, asserts that its top and the top of
/// its brace are inside one of the `carrier` volumes, and returns the height of its pad.
fn hang(carrier: &[Vec<Vec3>], hinge: Vec3, share: f32) -> f32 {
    let leg = corners(leg());
    let at = |p: &Vec3| hinge + Vec3::new(p.x, p.y * share, p.z);
    for top in leg.iter().filter(|p| p.y > -0.06).map(at) {
        let held = carrier.iter().any(|solid| inside(solid, top));
        assert!(held, "the leg at {share} hangs out of the ship at {top}");
    }
    leg.iter().map(|p| at(p).y).fold(f32::MAX, f32::min)
}

/// The legs hang from their hinge folded and down: the top of the strut and of the brace is
/// inside the fuselage, the wing or the pod at both ends of the travel, the pad of a leg that is
/// down is flat on the foot the flight uses, and a folded one lies flush with the belly.
#[test]
fn the_gear_reaches_the_ship_folded_and_down() {
    let carry = |side: f32| vec![wing(side), pod(1.9 * side, &MAIN)];
    let carriers = [fuselage().to_vec(), carry(-1.0), carry(1.0)];
    for (n, (foot, hinge)) in GEAR.iter().zip(hinges()).enumerate() {
        let mut volumes: Vec<Vec<Vec3>> = carriers[n].iter().cloned().map(corners).collect();
        if n == 0 {
            volumes.push(corners(pod(0.0, &NOSE)));
        }
        for (share, wanted) in [(FOLDED as f32, -0.6), (1.0, foot.y as f32)] {
            let pad = hang(&volumes, hinge, share);
            assert!((pad - wanted).abs() < 0.07, "pad at {pad}, wanted {wanted}");
        }
    }
}
