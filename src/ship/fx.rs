//! What the flight feels like from the seat: the values the sky draws its flight effects from
//! and the strength of the shake, both taken from the instruments.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::shake::{Buffet, FOLLOW, eased, roughness};
use super::telemetry::{ShipTelemetry, WarpState};
use crate::render::{CloudCrossing, FlightFx};
use crate::space::{Location, Travel};
use crate::view::Target;

/// Rate per second at which the warp effect follows the drive.
const WARP_RATE: f32 = 2.5;

/// Share of the full warp effect a fully spooled drive shows before it engages.
const SPOOL_SHARE: f32 = 0.2;

/// Returns how far the warp effect should be on, from 0 to 1, for a drive state.
pub fn warp_level(warp: WarpState) -> f32 {
    match warp {
        WarpState::Off => 0.0,
        WarpState::Spooling(ready) => SPOOL_SHARE * ready.clamp(0.0, 1.0) as f32,
        WarpState::Active(_) => 1.0,
    }
}

/// What the effects are read from and written to.
#[derive(SystemParam)]
pub struct Senses<'w> {
    /// Frame time.
    time: Res<'w, Time>,
    /// Whether the player is seated.
    location: Res<'w, Location>,
    /// The instruments.
    telemetry: Res<'w, ShipTelemetry>,
    /// How deep the view is in the cloud layer, when the sky is drawn.
    clouds: Option<Res<'w, CloudCrossing>>,
    /// The flight effects of the sky, when the sky is drawn.
    fx: Option<ResMut<'w, FlightFx>>,
    /// What the camera follows: the ship while the player is seated.
    target: Res<'w, Target>,
    /// The shake of the view.
    buffet: ResMut<'w, Buffet>,
}

/// Feeds the flight effects and the shake from the instruments while the player is seated,
/// and lets both fade to nothing on foot.
pub fn feed(mut senses: Senses) {
    let dt = senses.time.delta_secs();
    let seated = senses.location.travel == Travel::InShip;
    let gate = f32::from(u8::from(seated));
    let telemetry = *senses.telemetry;
    let pressure = telemetry.dynamic_pressure as f32 * gate;
    let depth = senses.clouds.as_deref().map_or(0.0, |clouds| clouds.depth);
    let rough = roughness(depth, pressure) * gate;
    let level = eased(senses.buffet.level, rough, FOLLOW, dt);
    senses.buffet.set_if_neq(Buffet { level });
    let Some(mut fx) = senses.fx else {
        return;
    };
    let next = FlightFx {
        dynamic_pressure: pressure,
        warp: eased(fx.warp, warp_level(telemetry.warp) * gate, WARP_RATE, dt),
        speed: telemetry.surface_speed as f32 * gate,
        nose: senses.target.forward * gate,
    };
    fx.set_if_neq(next);
}
