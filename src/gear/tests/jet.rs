//! The boot water jet: a top speed set by the liquid, a pump that never makes power, and nothing
//! in a frozen sea.

use bevy::prelude::*;

use super::{DT, Flight};
use crate::gear::jet::{boot_power, hull_drag, jet, liquid_density, pump};
use crate::gear::tune::{MAGMA_DENSITY, REF_GRAVITY, WATER_DENSITY};
use crate::gear::{FlightState, GearInput, Mode, step};
use crate::universe::{PlanetClass, PlanetRecord, Water};

/// Slowest and fastest top speed of the jet in home water, in metres per second.
const TOP_SPEED: (f32, f32) = (6.0, 12.0);

/// Returns an input holding the boots under the liquid of density `liquid`.
fn under(liquid: f32) -> GearInput {
    GearInput {
        boost: true,
        in_water: true,
        liquid,
        ..GearInput::default()
    }
}

/// Returns the speed a body reaches jetting straight ahead for `seconds` under `input`, the spool
/// stepped by the gear itself.
fn run(input: &GearInput, seconds: f32) -> f32 {
    let mut state = FlightState::default();
    let mut velocity = Vec3::ZERO;
    for _ in 0..(seconds / DT).round() as u32 {
        state = step(&state, input, DT);
        velocity = jet(velocity, Vec3::NEG_Z, &state, input, DT);
    }
    velocity.length()
}

/// Jetting in home water settles on a top speed in the documented range, where the thrust of
/// the pump meets the drag of the body.
#[test]
fn the_jet_settles_on_its_top_speed() {
    let water = under(WATER_DENSITY);
    let top = run(&water, 20.0);
    assert!(top > TOP_SPEED.0 && top < TOP_SPEED.1, "top speed {top}");
    assert!((run(&water, 30.0) - top).abs() < 0.05);
    let thrust = pump(boot_power(REF_GRAVITY), top, WATER_DENSITY);
    let drag = hull_drag(WATER_DENSITY) * top * top;
    assert!(
        (thrust - drag).abs() < 0.05 * drag,
        "{thrust} against {drag}"
    );
}

/// The pump never puts more power into the body than the boots deliver, and pushes less the
/// faster the body already goes.
#[test]
fn the_pump_never_makes_power() {
    let power = boot_power(REF_GRAVITY);
    let mut last = f32::INFINITY;
    for speed in [0.0, 2.0, 5.0, 10.0, 20.0, 40.0] {
        let thrust = pump(power, speed, WATER_DENSITY);
        assert!(thrust * speed <= power * 1.001, "{thrust} at {speed}");
        assert!(thrust > 0.0 && thrust < last);
        last = thrust;
    }
    assert_eq!(pump(power, 3.0, 0.0), 0.0);
    assert_eq!(pump(0.0, 3.0, WATER_DENSITY), 0.0);
}

/// A frozen sea or no liquid gives no push: the body only slows. Molten rock pushes.
#[test]
fn nothing_jets_without_a_liquid() {
    let frozen = under(0.0);
    assert_eq!(run(&frozen, 2.0), 0.0);
    let mut state = step(&FlightState::default(), &frozen, 1.0);
    assert_eq!(state.thrust, 0.0);
    state.thrust = 1.0;
    let coasting = jet(Vec3::NEG_Z * 4.0, Vec3::NEG_Z, &state, &frozen, DT);
    assert!(coasting.length() <= 4.0);
    assert!(run(&under(MAGMA_DENSITY), 5.0) > 1.0);
}

/// The boots spool up under water as they do in the air while the gear itself stays off.
#[test]
fn the_boots_spool_under_water() {
    let mut flight = Flight::new(Mode::Off, Vec3::ZERO);
    flight.run(&under(WATER_DENSITY), 0.5);
    assert_eq!(flight.state.mode, Mode::Off);
    assert_eq!(flight.state.thrust, 1.0);
}

/// The liquid comes from the planet: water where it is liquid, rock on a lava world, nothing
/// where it froze or boiled away.
#[test]
fn the_liquid_follows_the_planet() {
    let world = |class, water| PlanetRecord {
        class,
        water,
        ..PlanetRecord::TEMPERATE
    };
    assert_eq!(liquid_density(&PlanetRecord::TEMPERATE), WATER_DENSITY);
    assert_eq!(
        liquid_density(&world(PlanetClass::Lava, Water::None)),
        MAGMA_DENSITY
    );
    assert_eq!(liquid_density(&world(PlanetClass::Ice, Water::Ice)), 0.0);
    assert_eq!(
        liquid_density(&world(PlanetClass::Desert, Water::Steam)),
        0.0
    );
}
