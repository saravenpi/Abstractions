//! What the ship gives the sea under it: the push of its engines, its size and the air around it.

use bevy::prelude::*;

use super::body::Ship;
use super::sky::Skies;
use super::telemetry::ShipTelemetry;
use super::tune::SHIP_MASS;
use super::view::HULL_RADIUS;
use crate::effects::Churn;
use crate::gear::air_density;
use crate::universe::HOME_GRAVITY;
use crate::wind::thickness;

/// Sets the churn of the ship: its mass times the acceleration its crew feels pushes the air
/// down through a disc as wide as its hull, and nothing pushes while it rests.
pub fn feed(
    skies: Res<Skies>,
    telemetry: Res<ShipTelemetry>,
    mut churn: Single<&mut Churn, With<Ship>>,
) {
    let height = telemetry.altitude_sea as f32;
    let air = skies.planet().map_or(0.0, |planet| {
        air_density(planet.atmosphere_height as f32, height) * thickness(&planet.record) as f32
    });
    let load = SHIP_MASS * telemetry.g_force as f32 * HOME_GRAVITY as f32;
    churn.push = [load, 0.0][usize::from(telemetry.landed)];
    churn.size = 2.0 * HULL_RADIUS;
    churn.air = air;
    churn.jet = 0.0;
}
