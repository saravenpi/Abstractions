//! What the flight feels like from the seat: the values the sky draws its flight effects from
//! and the strength of the shake, both taken from the instruments.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Impact;
use super::shake::{Buffet, FOLLOW, eased, jolted, roughness};
use super::telemetry::{ShipTelemetry, WarpState};
use super::warp::fraction;
use crate::render::{CloudCrossing, FlightFx, Sequence};
use crate::space::{Location, Travel};
use crate::view::Target;

/// Share of the full warp effect a fully spooled drive shows before it engages.
const SPOOL_SHARE: f32 = 0.45;

/// Returns how far the warp effect should be on, from 0 to 1, for a drive state.
pub fn warp_level(warp: WarpState) -> f32 {
    match warp {
        WarpState::Off => 0.0,
        WarpState::Spooling(ready) => SPOOL_SHARE * ready.clamp(0.0, 1.0) as f32,
        WarpState::Active(_) => 1.0,
    }
}

/// Returns the speed of a drive as a share of the top warp speed, from 0 to 1: nothing until
/// it is engaged.
pub fn pace(warp: WarpState) -> f32 {
    match warp {
        WarpState::Active(speed) => fraction(speed) as f32,
        _ => 0.0,
    }
}

/// What the effects are read from and written to.
#[derive(SystemParam)]
pub struct Senses<'w, 's> {
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
    /// The warp sequence of the sky, when the sky is drawn: the drive shakes the view with it.
    sequence: Option<Res<'w, Sequence>>,
    /// What the camera follows: the ship while the player is seated.
    target: Res<'w, Target>,
    /// The shake of the view.
    buffet: ResMut<'w, Buffet>,
    /// The crashes of the frame, which jolt the view.
    impacts: MessageReader<'w, 's, Impact>,
}

/// Feeds the flight effects and the shake from the instruments while the player is seated,
/// jolts the view on a crash, and lets both fade to nothing on foot.
pub fn feed(mut senses: Senses) {
    let dt = senses.time.delta_secs();
    let seated = senses.location.travel == Travel::InShip;
    let gate = f32::from(u8::from(seated));
    let telemetry = *senses.telemetry;
    let pressure = telemetry.dynamic_pressure as f32 * gate;
    let depth = senses.clouds.as_deref().map_or(0.0, |clouds| clouds.depth);
    let tremor = senses.sequence.as_deref().map_or(0.0, Sequence::tremor);
    let rough = (roughness(depth, pressure) * gate).max(tremor);
    let level = eased(senses.buffet.level, rough, FOLLOW, dt);
    let hit = senses
        .impacts
        .read()
        .map(|impact| impact.speed)
        .reduce(f64::max);
    let jolt = jolted(senses.buffet.jolt, hit.filter(|_| seated), dt);
    senses.buffet.set_if_neq(Buffet { level, jolt });
    let Some(mut fx) = senses.fx else {
        return;
    };
    let next = FlightFx {
        dynamic_pressure: pressure,
        warp: warp_level(telemetry.warp) * gate,
        pace: pace(telemetry.warp) * gate,
        speed: telemetry.surface_speed as f32 * gate,
        nose: senses.target.forward * gate,
    };
    fx.set_if_neq(next);
}
