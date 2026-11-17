//! The ship as the companion senses it: the seat, the climb and the clouds, from the ship's own signals.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::render::CloudCrossing;
use crate::ship::ShipTelemetry;
use crate::space::Frame;

/// Depth in the cloud layer, from 0 to 1, beyond which the clouds count as being crossed.
const CLOUD_DEPTH: f32 = 0.1;

/// What the ship reports this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Flight {
    /// True while the player is seated in the ship.
    pub seated: bool,
    /// Height of the flying ship above sea level in meters, or nothing while it rests on the
    /// ground or is away from a planet surface.
    pub height: Option<f64>,
    /// How deep the view is in the cloud layer, from 0 to 1.
    pub cloud: f32,
}

/// Returns the depth under water, the climb rate and whether the clouds are being crossed.
///
/// `eyes` is the altitude of the player's eyes, `was` the height the ship reported on the last
/// frame and `flight` what it reports now. Climbing and clouds only count while seated: the
/// climb is the change of the ship's own altimeter and the clouds are the layer the sky draws.
pub fn read(eyes: Option<f32>, was: Option<f64>, flight: &Flight, dt: f32) -> (f32, f32, bool) {
    let depth = eyes.map_or(0.0, |height| (-height).max(0.0));
    let climbed = flight.height.zip(was).map_or(0.0, |(now, was)| now - was);
    let climb = climbed as f32 / dt.max(1e-4);
    (
        depth,
        if flight.seated { climb } else { 0.0 },
        flight.seated && flight.cloud > CLOUD_DEPTH,
    )
}

/// The ship as the companion senses it.
#[derive(SystemParam)]
pub struct Aboard<'w> {
    /// The instruments of the ship.
    telemetry: Option<Res<'w, ShipTelemetry>>,
    /// How deep the view is in the cloud layer, when the sky is drawn.
    clouds: Option<Res<'w, CloudCrossing>>,
}

impl Aboard<'_> {
    /// Returns what the ship reports, with the player `seated` in it or not.
    pub fn flight(&self, seated: bool) -> Flight {
        let flying = |t: &&ShipTelemetry| !t.landed && matches!(t.frame, Frame::Surface(_));
        let telemetry = self.telemetry.as_deref().filter(flying);
        Flight {
            seated,
            height: telemetry.map(|telemetry| telemetry.altitude_sea),
            cloud: self.clouds.as_deref().map_or(0.0, |clouds| clouds.depth),
        }
    }
}
