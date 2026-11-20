//! Tests of the flow of a wound: it grows with the cut and the pressure, falls to nothing at death
//! or when it clots, a spear plugs it, and a drop that lands lays a stain of the species.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::drain::advance;
use super::stain::drip;
use super::tune::{EMIT, PLUG, TRAIL};
use super::wound::{Circuit, Cut, Wound};
use crate::life::blood::tune::LETHAL;

/// Returns the circuit of a body of 100 kg with 16 kPa in its arteries.
fn circuit() -> Circuit {
    Circuit {
        mass: 100.0,
        pressure: 16_000.0,
        volume: 6.6e-3,
        beat: 1.3,
    }
}

/// Returns a fresh cut `width` metres wide, a major one when `major`.
fn cut(width: f32, major: bool) -> Cut {
    Cut {
        part: 0,
        at: Vec3::ZERO,
        normal: Vec3::Y,
        width,
        depth: 0.02,
        major,
        age: 0.0,
        owed: 0.0,
        plugged: false,
    }
}

/// The flow grows with the width of the cut and the pressure, and an artery outruns a seep.
#[test]
fn flow_grows_with_size_and_pressure() {
    let c = circuit();
    assert!(c.rate(&cut(0.02, true), 0.0, false) > c.rate(&cut(0.005, true), 0.0, false));
    let strong = Circuit {
        pressure: 32_000.0,
        ..c
    };
    assert!(strong.rate(&cut(0.02, true), 0.0, false) > c.rate(&cut(0.02, true), 0.0, false));
    assert!(c.rate(&cut(0.02, true), 0.0, false) > 10.0 * c.rate(&cut(0.1, false), 0.0, false));
}

/// The flow is none at the lethal loss and after a shallow wound clotted for long.
#[test]
fn flow_stops_at_death_and_when_clotted() {
    let c = circuit();
    let dead = c.volume * LETHAL;
    assert_eq!(c.rate(&cut(0.02, true), dead, false), 0.0);
    assert!(c.rate(&cut(0.02, true), 0.5 * dead, false) < c.rate(&cut(0.02, true), 0.0, false));
    let old = Cut {
        age: 5000.0,
        ..cut(0.1, false)
    };
    assert_eq!(c.rate(&old, 0.0, false), 0.0);
}

/// A spear in the wound cuts its flow to the plugged share.
#[test]
fn a_plugged_wound_flows_less() {
    let c = circuit();
    let open = c.rate(&cut(0.02, true), 0.0, false);
    let plugged = c.rate(&cut(0.02, true), 0.0, true);
    assert!(plugged < open && (plugged / open - PLUG).abs() < 1e-3);
}

/// Advancing a wound throws drops, loses blood by the flow, never more than the cap of drops a
/// frame, and closes a wound that clotted.
#[test]
fn advancing_throws_drops_and_closes() {
    let c = circuit();
    let mut wound = Wound::default();
    wound.add(cut(0.02, true));
    let mut drops = 0;
    let mut lost = 0.0;
    for _ in 0..120 {
        let mut frame = 0;
        let (l, mean) = advance(&mut wound, &c, 1.0 / 60.0, |_| false, |_, _| frame += 1);
        assert!(frame <= EMIT && mean > 0.0);
        drops += frame;
        lost += l;
    }
    assert!(drops > 10 && lost > 0.0);
    let mut shallow = Wound::default();
    shallow.add(Cut {
        age: 5000.0,
        ..cut(0.1, false)
    });
    advance(&mut shallow, &c, 1.0 / 60.0, |_| false, |_, _| {});
    assert!(!shallow.open());
}

/// A drop that lands lays a mark of its species, as wide as its film, along its heading.
#[test]
fn a_landed_drop_lays_a_stain_of_its_species() {
    let up = DVec3::Y;
    let mark = drip(up, DVec3::new(1.0, -3.0, 0.0), (7, 0.0062), 12.0);
    assert_eq!(mark.species, 7);
    assert_eq!(mark.time, 12.0);
    assert!(mark.facing.dot(up).abs() < 1e-9 && mark.facing.x > 0.99);
    assert!((TRAIL[0]..=TRAIL[1]).contains(&mark.size));
}
