//! Tests of the spot: distance, air, delay, vacuum, water and the ground on the way.

use bevy::prelude::*;

use super::air::Air;
use super::listen::{Listener, hidden};
use super::spot::{Between, Origin, Spot, spot};
use super::tune::{CROSS, STEP, WATER_SOUND};
use crate::universe::PlanetRecord;

/// Returns the spot of a source of reach 1 metre `metres` ahead of a listener in `air`.
fn ahead(air: Air, metres: f32) -> Spot {
    let ear = Listener {
        air,
        ..Listener::default()
    };
    spot(
        &ear,
        Origin::At(Vec3::NEG_Z * metres, 1.0),
        Between::default(),
    )
}

/// Returns the air at sea level of the home world with its pressure set to `pressure`.
fn thinned(pressure: f64) -> Air {
    Air::sea(&PlanetRecord {
        pressure,
        ..PlanetRecord::TEMPERATE
    })
}

/// Doubling the distance halves the pressure, 6 dB, and the air takes more of the high end.
#[test]
fn doubling_the_distance_drops_six_decibels_and_dulls() {
    let home = Air::home();
    for metres in [5.0, 20.0, 80.0, 300.0] {
        let (near, far) = (ahead(home, metres), ahead(home, 2.0 * metres));
        let drop = 20.0 * (near.air / far.air).log10();
        assert!((drop - 6.02).abs() < 0.05, "{metres} m: {drop} dB");
        assert!(far.corner < near.corner, "{metres} m");
    }
    assert_eq!(ahead(home, 0.5).air, 1.0);
}

/// The corner of the air falls with the distance and with thinner air.
#[test]
fn the_corner_falls_with_distance_and_thinner_air() {
    let (home, thin) = (Air::home(), thinned(0.1));
    for metres in [10.0, 50.0, 200.0, 1000.0] {
        assert!(
            ahead(thin, metres).corner < ahead(home, metres).corner,
            "{metres} m"
        );
    }
    let corners: Vec<f32> = [10.0, 30.0, 100.0, 300.0, 1000.0]
        .map(|m| ahead(home, m).corner)
        .to_vec();
    assert!(
        corners.windows(2).all(|pair| pair[1] < pair[0]),
        "{corners:?}"
    );
    let km = ahead(home, 1000.0).corner;
    assert!(km > 400.0 && km < 1100.0, "{km} Hz at a kilometre");
}

/// The delay is the distance over the speed of sound of the air, and thin air is quieter.
#[test]
fn the_delay_is_the_distance_over_the_speed_of_sound() {
    let home = Air::home();
    let spot = ahead(home, 100.0);
    assert!((spot.delay - 100.0 / home.sound).abs() < 1.0e-5);
    assert!((home.sound - 340.0).abs() < 5.0, "{}", home.sound);
    let thin = ahead(thinned(0.1), 100.0);
    assert!(thin.air < 0.15 * spot.air && thin.air > 0.05 * spot.air);
}

/// In a vacuum nothing comes through the air, and what the body holds comes through it alone.
#[test]
fn a_vacuum_carries_only_what_the_body_touches() {
    let ear = Listener {
        air: Air::VACUUM,
        ..Listener::default()
    };
    let feet = Vec3::NEG_Y * STEP.0;
    let far = spot(&ear, Origin::At(Vec3::X * 5.0, 1.0), Between::default());
    let own = spot(&ear, Origin::Held(feet, STEP.0, STEP.1), Between::default());
    assert!(far.air == 0.0 && far.body == 0.0 && !far.heard());
    assert!(own.air == 0.0 && own.body == STEP.1 && own.heard());
    assert_eq!((own.delay, own.gap), (0.0, 0.0));
}

/// Under water sound is faster and far quieter from above, and a flat sound is flat everywhere.
#[test]
fn water_is_faster_and_the_surface_keeps_little() {
    let ear = Listener {
        wet: true,
        ..Listener::default()
    };
    let at = Origin::At(Vec3::NEG_Z * 30.0, 1.0);
    let under = spot(
        &ear,
        at,
        Between {
            wet: true,
            hidden: false,
        },
    );
    let above = spot(&ear, at, Between::default());
    assert!((under.delay - 30.0 / WATER_SOUND).abs() < 1.0e-5);
    assert!(above.air < CROSS.0 * under.air + 1.0e-6 && above.corner <= CROSS.1);
    assert_eq!(spot(&ear, Origin::Flat, Between::default()), Spot::FLAT);
}

/// A hill between the listener and a source hides it, and flat ground does not.
#[test]
fn a_hill_hides_and_flat_ground_does_not() {
    let (from, to) = (Vec3::new(0.0, 1.7, 0.0), Vec3::new(100.0, 1.0, 0.0));
    let flat = |p: Vec3| p.y;
    let hill = |p: Vec3| p.y - 20.0 * (-((p.x - 50.0) / 10.0).powi(2)).exp();
    assert!(!hidden(flat, from, to) && hidden(hill, from, to));
    let ear = Listener::default();
    let open = spot(&ear, Origin::At(to, 1.0), Between::default());
    let behind = spot(
        &ear,
        Origin::At(to, 1.0),
        Between {
            wet: false,
            hidden: true,
        },
    );
    assert!(behind.air < open.air && behind.corner < open.corner);
}
