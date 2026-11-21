//! How far the air lets the focus see: what arrives beyond it needs no fade.

use std::f64::consts::FRAC_PI_2;

use bevy::math::DVec3;

use crate::render;
use crate::world::Globe;
use crate::world::globe::ChunkId;
use crate::world::globe::far::SIGHT;

/// Heights above the ground in metres between which the detailed ground stops being all the
/// focus looks at: on foot a gap in it shows, a few hundred metres up the coarse planet carries
/// the view. The flight tests weigh the gaps of the detailed ground by it.
#[cfg(test)]
const OPEN: (f64, f64) = (15.0, 300.0);

/// Reach of that measure in horizon distances once open.
#[cfg(test)]
const HORIZON_SHARE: f64 = 2.5;

/// Share of the reach of the detailed ground within which a gap in it shows.
#[cfg(test)]
const FOG_END: f64 = 0.9;

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
    /// Returns the metres the focus is taken down to be as high over the sea level sphere, where the
    /// nodes of the quadtree lie, as it is over the ground: a focus in a basin or on a mountain is then
    /// as near its nodes as one on a plain.
    pub fn drop(&self) -> f64 {
        self.altitude - self.clearance.max(0.0)
    }

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
    /// Returns the eye of a focus at planet position `at` over `globe`: it sees as far as the
    /// clear air of the planet lets it, by the aerial perspective the camera draws, and at most
    /// [`SIGHT`] planet radii. Thin air lets it see farther and thick air less.
    pub fn of(globe: &Globe, at: DVec3) -> Self {
        let height = Height::of(globe, at);
        let top = crate::universe::heights(globe.record(), globe.gravity(), globe.radius())[0];
        let air = render::Air::of(globe.record());
        let haze = render::Haze::of(&air, top as f32, &render::WeatherLook::CALM);
        let ground = (height.altitude - height.clearance.max(0.0)) as f32;
        let seen = f64::from(render::sight(haze.toward(height.altitude as f32, ground)));
        Self {
            at: at - at.normalize_or(DVec3::Y) * height.drop(),
            radius: globe.radius(),
            veil: seen.min(SIGHT * globe.radius()),
        }
    }

    /// Returns true when the nearest part of node `id` lies within that distance.
    pub fn sees(&self, id: ChunkId) -> bool {
        let width = self.radius * FRAC_PI_2 / (1u64 << id.level) as f64;
        let middle = id.dir(0.5, 0.5) * self.radius;
        middle.distance(self.at) - width * HALF_DIAGONAL < self.veil
    }
}

/// Returns the distance in metres within which a gap in the detailed ground shows to a focus at
/// `height`, when that ground reaches `ground` metres: on foot nine tenths of it, and beyond the
/// horizon once the focus has climbed.
#[cfg(test)]
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

    /// The eye sees a node whose near edge is within its sight, not one beyond it.
    #[test]
    fn eye_sees_what_is_within_its_sight() {
        let leaf = crate::world::globe::quadtree::leaf_at(DVec3::Y, 7);
        let along = |steps: u32| ChunkId {
            x: leaf.x + steps,
            ..leaf
        };
        let eye = Eye {
            at: leaf.dir(0.5, 0.5) * 6000.0,
            radius: 6000.0,
            veil: 207.0,
        };
        assert!(eye.sees(leaf) && eye.sees(along(2)));
        assert!(!eye.sees(along(6)));
    }

    /// Through the air of a planet the eye sees the coarse ground far past the detailed ground,
    /// kilometres away, and never past the reach the world draws at all.
    #[test]
    fn the_eye_sees_through_the_air_to_the_far_ground() {
        let globe = Globe::new(7, 6000.0, 9.81);
        let leaf = crate::world::globe::quadtree::leaf_at(DVec3::Y, 7);
        let eye = Eye::of(&globe, leaf.dir(0.5, 0.5) * 6002.0);
        assert!(
            eye.veil > 2000.0 && eye.veil <= SIGHT * 6000.0,
            "{}",
            eye.veil
        );
        let far = ChunkId {
            x: leaf.x + 6,
            ..leaf
        };
        assert!(eye.sees(leaf) && eye.sees(far));
    }
}
