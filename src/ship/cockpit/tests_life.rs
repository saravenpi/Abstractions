//! Life tests: the needles sweep with inertia and the loose strap hangs along the pull of the
//! frame.

use bevy::prelude::*;

use super::Cockpit;
use super::strap::{Strap, hang};
use super::tune::{SETTLE, SLACK, SWING};

/// A needle follows what it reads with a time constant of one over `SETTLE` seconds: it has
/// covered 63 percent of a step after that time, and never jumps. A bar with nothing to read
/// goes dark at once, and comes back from zero.
#[test]
fn a_needle_sweeps_with_inertia() {
    let mut cockpit = Cockpit::default();
    cockpit.read.bars[0] = Some(1.0);
    cockpit.gauges[0] = Some(0.0);
    let frames = (60.0 / SETTLE) as usize;
    let mut last = 0.0;
    for _ in 0..frames {
        cockpit.step(1.0 / 60.0);
        let now = cockpit.gauges[0].unwrap_or(-1.0);
        assert!(now >= last && now - last < 0.2);
        last = now;
    }
    assert!((last - 0.63).abs() < 0.1, "{last}");
    cockpit.read.bars[0] = None;
    cockpit.step(1.0 / 60.0);
    assert_eq!(cockpit.gauges[0], None);
    cockpit.read.bars[0] = Some(1.0);
    cockpit.step(1.0 / 60.0);
    assert_eq!(cockpit.gauges[0], Some(0.0));
}

/// The strap hangs straight down under the pull of the ground, swings aft when the ship
/// pulls forward, floats upright in free fall, never leaves a slope of one, and swings past
/// its place before it settles there.
#[test]
fn the_strap_hangs_along_the_pull() {
    let ground = Vec3::Y * 9.81;
    assert!(hang(ground).length() < 1.0e-6 && hang(Vec3::ZERO) == Vec2::ZERO);
    assert!(hang(Vec3::new(0.0, 9.81, -9.81)).y > 0.99);
    assert!(hang(Vec3::new(1.0e6, 0.0, 0.0)).length() <= 1.0 + 1.0e-6);
    assert!(hang(Vec3::new(0.0, SLACK, -SLACK)).y > 0.99);
    let pulled = Vec3::new(0.0, 9.81, -4.905);
    let (mut strap, mut peak) = (Strap::default(), 0.0_f32);
    for _ in 0..600 {
        strap = strap.stepped(pulled, 1.0 / 60.0);
        peak = peak.max(strap.tilt.y);
    }
    assert!(peak > 0.55 && (strap.tilt.y - 0.5).abs() < 1.0e-2, "{peak}");
    assert!(SWING.1 < 1.0);
    let thrown = Strap::default().stepped(pulled * 1.0e6, 10.0);
    assert!(thrown.tilt.is_finite() && thrown.tilt.length() < 1.0);
}
