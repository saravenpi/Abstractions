//! The heat of the ship in fast flight through air, whichever way it flies: what it flies through
//! goes to [`Heat`], which the sheath, the cabin, the shake and the roar follow.

use bevy::prelude::*;

use super::telemetry::ShipTelemetry;
use crate::render::{Craft, Heat, LocalSystem, MIN_SPEED, Vantage, entry};
use crate::space::{Location, Travel};
use crate::view::Target;

/// Run condition: the player is seated in a ship that flies fast enough, through air, for its
/// nose to heat, or whose nose still glows or cools after the air: the sheath follows the ship
/// until the skin is dark.
pub fn hot(
    location: Option<Res<Location>>,
    telemetry: Res<ShipTelemetry>,
    heat: Option<Res<Heat>>,
) -> bool {
    let seated = location.is_some_and(|location| location.travel == Travel::InShip);
    let heating = telemetry.dynamic_pressure > 0.0 && telemetry.surface_speed >= MIN_SPEED;
    let cooling = heat.is_some_and(|heat| heat.glow.heat > 0.0 || heat.glow.kelvin > 0.0);
    seated && (heating || cooling)
}

/// Returns the density of the air in kilograms per cubic metre from the dynamic pressure
/// `0.5 * rho * v^2` in pascals and the speed in metres per second, the air the drag uses.
fn density(pressure: f64, speed: f64) -> f64 {
    2.0 * pressure / (speed * speed).max(1.0)
}

/// Offers the heat of the ship: the glow of its speed through the air, down or up alike.
pub fn offer(
    telemetry: Res<ShipTelemetry>,
    target: Res<Target>,
    sky: (Option<Res<Vantage>>, Option<Res<LocalSystem>>),
    heat: Option<ResMut<Heat>>,
) {
    let (Some(vantage), Some(local), Some(mut heat)) = (sky.0, sky.1, heat) else {
        return;
    };
    let gas = local.gas(&vantage);
    let air = density(telemetry.dynamic_pressure, telemetry.surface_speed);
    let glow = entry(air, telemetry.surface_speed, gas);
    heat.offer(
        glow,
        Craft {
            centre: target.position,
            heading: target.forward,
            radius: target.radius,
            roof: target.up,
        },
    );
}
