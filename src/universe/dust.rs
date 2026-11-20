//! The dust of a galaxy: where its starlight is absorbed on the way to a camera. Dust lies in
//! the plane of the disc, in lanes along the inner edge of the arms of a spiral, and in patches
//! in an irregular cloud. A ball of old stars holds almost none.

use bevy::math::DVec3;

use super::galaxy::Shape;
use super::hash;
use super::place::{Place, smooth};
use super::spec::GalaxyKind;

/// Optical depth per game light-year at the peak of a lane.
const DUST: f64 = 0.004;

/// Radians the lanes lie ahead of the stars of the arm, on its inner edge.
const LANE_SHIFT: f64 = 0.35;

/// Share of the dust of a lane that lies between the lanes of a spiral, in the plain disc.
const BETWEEN: f64 = 0.15;

/// Share of the dust of a lane that a ball of stars holds at its centre.
const BALL: f64 = 0.1;

impl Shape {
    /// Returns the dust at `place`, as the optical depth per game light-year of a ray through it: 0 in
    /// the void and above the disc.
    pub fn dust_at(&self, place: &Place) -> f64 {
        let Place { q, r, alive, .. } = *place;
        let layer = |scale: f64| (-0.5 * (q.y / (scale * self.thick)).powi(2) - r / 0.3).exp();
        let found = match self.kind {
            GalaxyKind::Spiral => layer(0.8) * smooth(1.0, 0.9, r) * self.lanes(place),
            GalaxyKind::Elliptical => BALL * (-r.hypot(q.y / 0.7) / 0.18).exp(),
            GalaxyKind::Irregular => layer(3.0) * smooth(1.0, 0.8, r) * self.patches(q),
        };
        alive * DUST * found
    }

    /// Returns the share of the peak of a lane the arms of a spiral hold at `place`: sharp lanes
    /// along the arms and a little between them.
    fn lanes(&self, place: &Place) -> f64 {
        let spoke = place.turn - self.swirl * place.r - LANE_SHIFT;
        let wave = 0.5 + 0.5 * (f64::from(self.arms) * spoke).cos();
        BETWEEN + (1.0 - BETWEEN) * wave.powi(4)
    }

    /// Returns the share of the peak of a patch an irregular cloud holds at `q` radii from the
    /// centre, the patches hashed on a coarse grid.
    fn patches(&self, q: DVec3) -> f64 {
        let cell = hash::cell(self.seed ^ 0xD057, (q * 10.0).floor().as_ivec3());
        let patch = cell as f64 / u64::MAX as f64;
        patch * patch
    }
}
