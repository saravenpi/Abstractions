//! Lift nozzle tests: they fade out smoothly as the ship gathers speed along its nose, and the
//! ships of other players hover on them by the same rule, from what their records carry.

use super::state::{held, hovering};
use super::tune::CARRY;
use crate::ship::cockpit::{Cockpit, Lamp};
use crate::ship::tune::LIFT_ACCEL;
use crate::universe::STANDARD_GRAVITY;

/// The share of the lift nozzles is all of it at rest and backwards, none past the second
/// speed of [`CARRY`], and it never steps by more than 2 percent for each 0.1 m/s on the way:
/// the nozzles fade, they do not pop.
#[test]
fn the_lift_nozzles_fade_with_forward_speed() {
    assert_eq!((hovering(-5.0), hovering(0.0)), (1.0, 1.0));
    assert_eq!(hovering(CARRY.1), 0.0);
    let shares: Vec<f64> = (0..=600)
        .map(|tenth| hovering(f64::from(tenth) * 0.1))
        .collect();
    assert!(shares.windows(2).all(|pair| pair[1] <= pair[0]));
    assert!(shares.windows(2).all(|pair| pair[0] - pair[1] < 0.02));
}

/// A ship of another player that hangs still holds its weight on its lift nozzles, one that
/// flies fast does not, and neither does one that is landed, in warp or away from a planet.
#[test]
fn a_remote_ship_hovers_on_its_lift_nozzles() {
    let weight = (STANDARD_GRAVITY / LIFT_ACCEL) as f32;
    let mut cockpit = Cockpit::default();
    cockpit.read.speed = Some(0.0);
    assert!((held(&cockpit, STANDARD_GRAVITY) - weight).abs() < 1.0e-6);
    assert_eq!(held(&cockpit, 0.0), 0.0);
    cockpit.read.speed = Some(2.0 * CARRY.1);
    assert_eq!(held(&cockpit, STANDARD_GRAVITY), 0.0);
    cockpit.read.speed = None;
    assert_eq!(held(&cockpit, STANDARD_GRAVITY), 0.0);
    cockpit.read.speed = Some(0.0);
    cockpit.read.lamps[Lamp::Landed as usize] = true;
    assert_eq!(held(&cockpit, STANDARD_GRAVITY), 0.0);
}
