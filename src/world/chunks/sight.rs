//! How far the fog lets the focus see: what arrives beyond it needs no fade.

use std::f64::consts::FRAC_PI_2;

use bevy::math::DVec3;

use crate::render;
use crate::world::Globe;
use crate::world::globe::ChunkId;
use crate::world::globe::far::SIGHT;

/// Heights above the ground in metres between which the fog opens from the edge of the detailed
/// ground to the horizon, read from `render::fog`, which draws the fog.
const OPEN: (f64, f64) = (render::FOG_OPEN.0 as f64, render::FOG_OPEN.1 as f64);

/// Fog reach in horizon distances once open.
const HORIZON_SHARE: f64 = render::HORIZON_SHARE as f64;

/// Share of the fog reach at which the linear fog hides everything.
const FOG_END: f64 = render::FOG_END as f64;

/// Where the focus is over the planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Height {
    /// Height above sea level in metres.
    pub altitude: f64,
    /// Height above the ground or the sea under the focus, in metres.
    pub clearance: f64,
    /// Sea level radius of the planet in metres.
    pub radius: f64,
}

impl Height {
    /// Returns how high planet position `focus` is over the sea and over the ground of `globe`.
    pub fn of(globe: &Globe, focus: DVec3) -> Self {
        let altitude = focus.length() - globe.radius();
        let ground = globe
            .height(focus.normalize_or(DVec3::Y))
            .max(globe.sea_level());
        Self {
            altitude,
            clearance: altitude - f64::from(ground),
            radius: globe.radius(),
        }
    }
}

/// Share of its width a node reaches from its middle towards a corner, rounded up.
const HALF_DIAGONAL: f64 = 0.75;

/// What the focus can see through the fog: what arrives there fades in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    at: DVec3,
    radius: f64,
    veil: f64,
}

impl Eye {
    /// Returns the eye of a focus at planet position `at` over `globe`, where the fog on foot
    /// lets it see `ground` metres. Thin air lets it see farther and thick air less, as the
    /// drawn fog does.
    pub fn of(globe: &Globe, at: DVec3, ground: f64) -> Self {
        let air = render::Air::of(globe.record());
        let thick = f64::from((air.share() * air.haze()).max(render::THIN_AIR));
        let veil = (veil(ground, &Height::of(globe, at)) / thick).min(SIGHT * globe.radius());
        Self {
            at,
            radius: globe.radius(),
            veil,
        }
    }

    /// Returns true when the nearest part of node `id` lies within the fog distance.
    pub fn sees(&self, id: ChunkId) -> bool {
        let width = self.radius * FRAC_PI_2 / (1u64 << id.level) as f64;
        let middle = id.dir(0.5, 0.5) * self.radius;
        middle.distance(self.at) - width * HALF_DIAGONAL < self.veil
    }
}

/// Returns the distance in metres at which the fog hides everything from a focus at `height`,
/// when the fog on foot lets it see `ground` metres: on foot nine tenths of that, and beyond the
/// horizon once the focus has climbed.
pub fn veil(ground: f64, height: &Height) -> f64 {
    let up = height.altitude.max(0.0);
    let horizon = (up * (2.0 * height.radius + up)).sqrt();
    let x = ((height.clearance - OPEN.0) / (OPEN.1 - OPEN.0)).clamp(0.0, 1.0);
    let open = x * x * (3.0 - 2.0 * x);
    let far = ground.max(horizon * HORIZON_SHARE);
    (ground + (far - ground) * open) * FOG_END
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On foot the fog ends just short of the detailed ground, and it opens to beyond the
    /// horizon with height.
    #[test]
    fn veil_opens_with_height() {
        let at = |clearance: f64| Height {
            altitude: clearance + 5.0,
            clearance,
            radius: 6000.0,
        };
        assert!((veil(200.0, &at(2.0)) - 180.0).abs() < 1e-3);
        assert!((veil(200.0, &at(15.0)) - 180.0).abs() < 1e-3);
        assert!(veil(200.0, &at(30.0)) < 230.0);
        assert!(veil(200.0, &at(1000.0)) > 8000.0);
        let heights = (0..80).map(|i| veil(200.0, &at(f64::from(i) * 5.0)));
        assert!(heights.collect::<Vec<_>>().windows(2).all(|w| w[0] <= w[1]));
    }

    /// The eye sees a node whose near edge is within the fog, not one beyond it.
    #[test]
    fn eye_sees_what_is_within_the_fog() {
        let globe = Globe::new(7, 6000.0, 9.81);
        let leaf = crate::world::globe::quadtree::leaf_at(DVec3::Y, 7);
        let at = leaf.dir(0.5, 0.5) * 6002.0;
        let along = |steps: u32| ChunkId {
            x: leaf.x + steps,
            ..leaf
        };
        let eye = Eye::of(&globe, at, 230.0);
        assert!(eye.sees(leaf) && eye.sees(along(2)));
        assert!(!eye.sees(along(6)));
    }
}
