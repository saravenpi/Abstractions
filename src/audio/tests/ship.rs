//! The voice of a ship: its loudness and its spectrum follow the throttle, the air, the
//! distance and the side of the canopy the listener is on.

use super::rig::{band, heard, parked, rms, seated};
use crate::audio::ship::feed::Ch;
use crate::audio::ship::law::peak_hz;
use crate::audio::ship::tune::DIAMETERS;

/// More throttle is louder, and it moves the roar down: a wide open engine makes a wider jet,
/// whose noise peaks lower.
#[test]
fn throttle_makes_the_jet_louder_and_deeper() {
    let low = heard(parked().with(Ch::Thrust, 0.2), 4.0);
    let full = heard(parked().with(Ch::Thrust, 1.0), 4.0);
    assert!(rms(&full) > 1.5 * rms(&low), "{} {}", rms(&full), rms(&low));
    assert!(rms(&low) > 0.002);
    let (deep, thin) = (
        peak_hz(340.0, DIAMETERS.0, 1.0),
        peak_hz(340.0, DIAMETERS.0, 0.2),
    );
    assert!(deep < thin);
    let tilt = |samples: &[f32]| band(samples, deep) / band(samples, thin);
    assert!(
        tilt(&full) > 1.2 * tilt(&low),
        "{} {}",
        tilt(&full),
        tilt(&low)
    );
}

/// Thin air carries less of the jet, and no air carries nothing: outside a ship in a vacuum
/// there is exact silence, whatever its engines do.
#[test]
fn the_air_carries_the_sound() {
    let burning = parked().with(Ch::Thrust, 1.0).with(Ch::Lift, 1.0);
    let thick = heard(burning, 3.0);
    let thin = heard(burning.with(Ch::Air, 0.3), 3.0);
    let none = heard(burning.with(Ch::Air, 0.0).with(Ch::Rcs, 1.0), 3.0);
    assert!(
        rms(&thick) > 2.0 * rms(&thin),
        "{} {}",
        rms(&thick),
        rms(&thin)
    );
    assert!(rms(&thin) > 0.002);
    assert!(none.iter().all(|x| *x == 0.0));
}

/// In a vacuum the pilot still hears the ship, through the airframe: low and muffled.
#[test]
fn a_vacuum_leaves_the_airframe() {
    let burning = seated(parked().with(Ch::Thrust, 1.0).with(Ch::Air, 0.0));
    let inside = heard(burning, 4.0);
    assert!(rms(&inside) > 0.02, "{}", rms(&inside));
    assert!(band(&inside, 85.0) > 4.0 * band(&inside, 3000.0));
    let off = seated(parked().with(Ch::Power, 0.0).with(Ch::Air, 0.0));
    assert!(rms(&heard(off, 2.0)) > 0.001, "the cabin bed stays");
}

/// Distance makes a ship quieter and duller: the sound spreads, and the air takes its high end.
#[test]
fn distance_thins_and_dulls_the_sound() {
    let burning = parked().with(Ch::Thrust, 1.0);
    let near = heard(burning, 3.0);
    let far = heard(burning.with(Ch::Distance, 600.0), 3.0);
    assert!(rms(&near) > 4.0 * rms(&far), "{} {}", rms(&near), rms(&far));
    assert!(rms(&far) > 0.0);
    let bright = |samples: &[f32]| band(samples, 4000.0) / band(samples, 110.0);
    assert!(bright(&near) > 1.5 * bright(&far));
}

/// The canopy is a low-pass: the seat hears less of the high end than the air outside, and an
/// open canopy lets it back in.
#[test]
fn the_canopy_muffles_the_jet() {
    let burning = parked().with(Ch::Thrust, 0.6);
    let outside = heard(burning, 3.0);
    let shut = heard(seated(burning), 3.0);
    let open = heard(seated(burning).with(Ch::Canopy, 1.0), 3.0);
    let bright = |samples: &[f32]| band(samples, 3000.0) / band(samples, 100.0);
    assert!(bright(&outside) > 3.0 * bright(&shut));
    assert!(bright(&open) > 2.0 * bright(&shut));
    assert!(rms(&shut) > 0.01);
}

/// The ground under the lift nozzles makes them louder, and the warp drive is heard from the seat.
#[test]
fn the_ground_and_the_drive_are_heard() {
    let hover = parked().with(Ch::Lift, 1.0);
    let low = heard(hover.with(Ch::Height, 1.0), 3.0);
    let high = heard(hover.with(Ch::Height, 200.0), 3.0);
    assert!(rms(&low) > 1.3 * rms(&high), "{} {}", rms(&low), rms(&high));
    let quiet = seated(parked().with(Ch::Air, 0.0));
    let warp = heard(quiet.with(Ch::Warp, 1.0), 3.0);
    assert!(rms(&warp) > 1.5 * rms(&heard(quiet, 3.0)));
}
