//! Flight instruments: what the HUD, the audio and the sky read about the ship.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::frames::spin_rate;
use super::orbit::apsides;
use super::sim::Pace;
use super::sky::Skies;
use super::warp::{SPOOL_TIME, Warp};
use super::wells::nearest;
use crate::space::Frame;
use crate::universe::{BodyId, HOME_GRAVITY};

/// What the warp drive is doing, for display.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WarpState {
    /// The drive is off.
    #[default]
    Off,
    /// The drive spools. The value runs from 0 to 1 as it gets ready.
    Spooling(f64),
    /// The ship is in warp at this speed in metres per second.
    Active(f64),
}

/// Read-only instruments of the ship, refreshed every frame of play.
///
/// Heights and speeds are measured against the planet the ship is on, or in space against the
/// body whose lock sphere is nearest. Between the stars they keep their last values.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ShipTelemetry {
    /// Height of the landing gear above the ground or the sea in metres. Away from a surface it
    /// equals `altitude_sea`.
    pub altitude_ground: f64,
    /// Height of the hull centre above sea level in metres.
    pub altitude_sea: f64,
    /// Speed relative to the turning surface in metres per second. In space it equals `orbital_speed`.
    pub surface_speed: f64,
    /// Speed relative to the centre of the body in metres per second, in a frame that does not turn.
    pub orbital_speed: f64,
    /// Main engine lever from 0 to 1.
    pub throttle: f64,
    /// Dynamic pressure of the air on the hull in pascals.
    pub dynamic_pressure: f64,
    /// Acceleration the crew feels, in home planet gravities.
    pub g_force: f64,
    /// What the warp drive is doing.
    pub warp: WarpState,
    /// Highest point of the path above sea level in metres, while the ship flies a closed path.
    pub apoapsis: Option<f64>,
    /// Lowest point of the path above sea level in metres, negative when the path hits the
    /// ground, while the ship flies a closed path.
    pub periapsis: Option<f64>,
    /// Whether the ship rests on the ground or on the sea.
    pub landed: bool,
    /// Frame the ship is simulated in.
    pub frame: Frame,
    /// Damage from hard landings, from 0 to 1.
    pub damage: f64,
    /// Whether the flight assist is on.
    pub assist: bool,
}

impl Default for ShipTelemetry {
    fn default() -> Self {
        Self {
            altitude_ground: 0.0,
            altitude_sea: 0.0,
            surface_speed: 0.0,
            orbital_speed: 0.0,
            throttle: 0.0,
            dynamic_pressure: 0.0,
            g_force: 0.0,
            warp: WarpState::Off,
            apoapsis: None,
            periapsis: None,
            landed: true,
            frame: Frame::Surface(BodyId::default()),
            damage: 0.0,
            assist: true,
        }
    }
}

/// The ship measured against one body.
struct Fix {
    /// Position from the centre of the body in metres.
    pos: DVec3,
    /// Velocity relative to the centre in a frame that does not turn.
    orbital: DVec3,
    /// Velocity relative to the surface.
    ground: DVec3,
    /// Gravitational parameter of the body.
    mu: f64,
    /// Radius of the body in metres.
    radius: f64,
}

/// Returns the ship measured against the planet it is on or the nearest body of its system.
///
/// The bodies were sampled `lag` seconds before now, so each is moved on to where it is now.
fn fix(ship: &Ship, skies: &Skies, lag: f64) -> Option<Fix> {
    match ship.frame {
        Frame::Surface(_) => {
            let planet = skies.planet()?;
            let drift = (DVec3::Y * spin_rate(planet)).cross(ship.pos);
            let (mu, radius) = (planet.mu(), planet.radius);
            Some(Fix {
                pos: ship.pos,
                orbital: ship.vel + drift,
                ground: ship.vel,
                mu,
                radius,
            })
        }
        Frame::System(_) => {
            let well = nearest(&skies.wells, ship.pos)?;
            let centre = well.pos + well.vel * lag;
            let (pos, vel) = (ship.pos - centre, ship.vel - well.vel);
            Some(Fix {
                pos,
                orbital: vel,
                ground: vel,
                mu: well.mu,
                radius: well.radius,
            })
        }
        Frame::Galaxy(_) => None,
    }
}

/// Returns the warp state for display.
fn warp_state(warp: Warp) -> WarpState {
    match warp {
        Warp::Off => WarpState::Off,
        Warp::Spooling { left } => WarpState::Spooling((1.0 - left / SPOOL_TIME).clamp(0.0, 1.0)),
        Warp::Active { speed, .. } => WarpState::Active(speed),
    }
}

/// Refreshes the instruments from the ship and the last flight step.
pub fn update(
    skies: Res<Skies>,
    pace: Res<Pace>,
    ship: Single<&Ship>,
    mut telemetry: ResMut<ShipTelemetry>,
) {
    let mut next = ShipTelemetry {
        throttle: ship.throttle,
        dynamic_pressure: pace.report.pressure,
        g_force: pace.report.proper.length() / HOME_GRAVITY,
        warp: warp_state(ship.warp),
        landed: ship.landed,
        frame: ship.frame,
        damage: ship.damage,
        assist: ship.assist,
        apoapsis: None,
        periapsis: None,
        ..*telemetry
    };
    if let Some(fix) = fix(&ship, &skies, pace.lag) {
        let path = apsides(fix.pos, fix.orbital, fix.mu).filter(|_| !ship.landed);
        next.altitude_sea = fix.pos.length() - fix.radius;
        next.altitude_ground = pace.report.radar.min(next.altitude_sea);
        next.surface_speed = fix.ground.length();
        next.orbital_speed = fix.orbital.length();
        next.apoapsis = path.map(|(apo, _)| apo - fix.radius);
        next.periapsis = path.map(|(_, peri)| peri - fix.radius);
    }
    telemetry.set_if_neq(next);
}
