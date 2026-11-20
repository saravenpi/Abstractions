//! The heat of the ship in a fast fall through thick air: what it flies through goes to
//! [`Heat`], which the view, the sheath, the shake and the roar follow.

use bevy::prelude::*;

use super::telemetry::ShipTelemetry;
use crate::render::{Craft, Heat, LocalSystem, MIN_SPEED, Vantage, descent, entry};
use crate::space::{Location, Travel};
use crate::view::Target;

/// Run condition: the player is seated in a ship that flies fast enough, through air, for its
/// nose to heat.
pub fn hot(location: Option<Res<Location>>, telemetry: Res<ShipTelemetry>) -> bool {
    location.is_some_and(|location| location.travel == Travel::InShip)
        && telemetry.dynamic_pressure > 0.0
        && telemetry.surface_speed >= MIN_SPEED
}

/// Returns the density of the air in kilograms per cubic metre from the dynamic pressure
/// `0.5 * rho * v^2` in pascals and the speed in metres per second, the air the drag uses.
fn density(pressure: f64, speed: f64) -> f64 {
    2.0 * pressure / (speed * speed)
}

/// Offers the heat of the ship: the glow of its speed through the air, only for a fall.
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
    let sink = -vantage.velocity.dot(vantage.up);
    let glow = entry(air, telemetry.surface_speed, gas).scaled(descent(sink));
    let heading = vantage.velocity.normalize_or(target.forward);
    heat.offer(
        glow,
        Craft {
            centre: target.position,
            heading,
            radius: target.radius,
        },
    );
}
