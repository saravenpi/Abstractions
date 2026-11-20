//! The boot water jet: under a liquid the boots pump it and push the body where it looks.
//!
//! The boots deliver the same power as in the air. A pump of nozzle area `A` throwing liquid of
//! density `rho` at jet speed `u` past a body moving at `v` adds the power `rho A u (u^2 - v^2) / 2`
//! and gives the thrust of the momentum flux, `rho A u (u - v)`. The body drags through the
//! liquid as through air, with the liquid density in place of the air one, so the top speed is
//! where the thrust meets `rho Cd A v^2 / 2`. A frozen sea and a world without liquid give nothing.

use bevy::prelude::*;

use super::input::GearInput;
use super::state::FlightState;
use super::tune::{
    AERO, BODY_DRAG, BODY_MASS, MAGMA_DENSITY, NOZZLE_AREA, POWER_SPEED, PUMP_STEPS, SEA_DENSITY,
    THRUST_G, WATER_DENSITY,
};
use crate::universe::{PlanetClass, PlanetRecord, Water};

/// Returns the density in kilograms per cubic metre of the liquid of the sea of `record`: molten
/// rock on a lava world, water where it is liquid, and 0 where the sea is frozen or no liquid is.
pub fn liquid_density(record: &PlanetRecord) -> f32 {
    match (record.class, record.water) {
        (PlanetClass::Lava, _) => MAGMA_DENSITY,
        (PlanetClass::Ice, _) => 0.0,
        (_, Water::Liquid) => WATER_DENSITY,
        _ => 0.0,
    }
}

/// Returns the power of the boots in watts per kilogram of the body under `gravity`: the thrust
/// they give in the air, a share of the weight, at the speed they fly at.
pub fn boot_power(gravity: f32) -> f32 {
    THRUST_G * gravity.max(0.0) * POWER_SPEED
}

/// Returns the thrust in metres per second squared that `power` watts per kilogram give through
/// the nozzles in a liquid of density `liquid`, for a body moving at `speed` along the jet.
///
/// The jet speed solves `power = c u (u^2 - v^2) / 2` with `c` the liquid mass the nozzles take
/// in per metre of jet, per kilogram of body. It lies between `v` and `v + (2 power / c)^(1/3)`,
/// where the power is always reached, and is found by bisection.
pub fn pump(power: f32, speed: f32, liquid: f32) -> f32 {
    let c = liquid * NOZZLE_AREA / BODY_MASS;
    if c <= 0.0 || power <= 0.0 {
        return 0.0;
    }
    let v = speed.max(0.0);
    let added = |u: f32| 0.5 * c * u * (u * u - v * v);
    let (mut low, mut high) = (v, v + (2.0 * power / c).cbrt());
    for _ in 0..PUMP_STEPS {
        let mid = 0.5 * (low + high);
        match added(mid) < power {
            true => low = mid,
            false => high = mid,
        }
    }
    let u = 0.5 * (low + high);
    c * u * (u - v)
}

/// Returns the drag of the streamlined body in a liquid of density `liquid`, per metre: the
/// deceleration is this times the speed squared.
pub fn hull_drag(liquid: f32) -> f32 {
    AERO * BODY_DRAG * liquid.max(0.0) / SEA_DENSITY
}

/// Returns true while `input` asks the boots to jet: they are held under a liquid.
pub fn jetting(input: &GearInput) -> bool {
    input.in_water && input.boost && input.liquid > 0.0
}

/// Returns the velocity of a body jetting along the unit direction `aim` after `dt` seconds,
/// from `velocity`, with the spool of `state` and the liquid and gravity of `input`.
///
/// The thrust is added first, then the drag slows the result implicitly, so it never
/// overshoots and never adds speed. Nothing pushes while the boots are not held.
pub fn jet(velocity: Vec3, aim: Vec3, state: &FlightState, input: &GearInput, dt: f32) -> Vec3 {
    let held = f32::from(u8::from(jetting(input)));
    let power = boot_power(input.gravity) * state.power() * held;
    let thrust = pump(power, velocity.dot(aim), input.liquid);
    let free = velocity + aim * (thrust * dt);
    free / (1.0 + hull_drag(input.liquid) * free.length() * dt)
}
