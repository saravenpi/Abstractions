//! The heat of the ship in fast flight through air: descending entries compress and ionise the
//! air ahead of the hull, while climbing exits shed plasma and skin heat and keep dynamic pressure.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::telemetry::ShipTelemetry;
use super::view::HULL_RADIUS;
use crate::render::{
    Craft, Gas, Glow, Heat, LocalSystem, MIN_SPEED, Vantage, entry, glowing, smoothstep,
    temperature,
};
use crate::space::{Frame, Location, Travel};
use crate::view::Target;

/// Sine of upward climb angle where atmospheric entry heating starts fading out and where it is gone.
const EXIT_BAND: (f32, f32) = (0.05, 0.25);

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

/// Returns the glow of a ship flying at `speed` with outward `radial_speed` through `air` of
/// `gas`: full Sutton-Graves entry heating while descending or level, attenuated to zero on steep
/// atmospheric exit while preserving dynamic `pressure`.
pub fn glow_for(air: f64, speed: f64, radial_speed: f64, gas: Gas) -> Glow {
    let raw = entry(air, speed, gas);
    let climb = (radial_speed / speed.max(1.0)).clamp(-1.0, 1.0) as f32;
    let share = 1.0 - smoothstep(EXIT_BAND.0, EXIT_BAND.1, climb);
    if share >= 1.0 {
        return raw;
    }
    let flux = raw.flux * share;
    let kelvin = if share <= 0.0 || flux <= 0.0 {
        0.0
    } else {
        temperature(f64::from(flux)) as f32
    };
    Glow {
        heat: raw.heat * share,
        pressure: raw.pressure,
        color: raw.color,
        flux,
        kelvin,
        shine: glowing(kelvin),
    }
}

/// Offers the heat of the ship: entry plasma and hot skin on descent or level dash, attenuated on
/// atmospheric exit while keeping dynamic pressure.
pub fn offer(
    (telemetry, ship): (Res<ShipTelemetry>, Single<&Ship>),
    target: Res<Target>,
    sky: (Option<Res<Vantage>>, Option<Res<LocalSystem>>),
    heat: Option<ResMut<Heat>>,
) {
    let (Some(vantage), Some(local), Some(mut heat)) = (sky.0, sky.1, heat) else {
        return;
    };
    let gas = local.gas(&vantage);
    let air = density(telemetry.dynamic_pressure, telemetry.surface_speed);
    let radial = match ship.frame {
        Frame::Surface(_) => ship.vel.dot(ship.pos.normalize_or(DVec3::Y)),
        _ => 0.0,
    };
    let glow = glow_for(air, telemetry.surface_speed, radial, gas);
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

/// Locks a live [`Heat`] pose to the ship's propagated world transform so rebases, frame switches
/// and cockpit shake never separate the sheath from the hull.
pub fn align(
    location: Option<Res<Location>>,
    ships: Query<&GlobalTransform, With<Ship>>,
    heat: Option<ResMut<Heat>>,
) {
    let seated = location.is_some_and(|location| location.travel == Travel::InShip);
    let Some(mut heat) = heat.filter(|heat| seated && heat.live()) else {
        return;
    };
    let Ok(hull) = ships.single() else {
        return;
    };
    let craft = Craft {
        centre: hull.translation(),
        heading: hull.forward().as_vec3(),
        radius: HULL_RADIUS,
        roof: hull.up().as_vec3(),
    };
    heat.craft = craft;
    if let Some((_, offered)) = &mut heat.offered {
        *offered = craft;
    }
}
