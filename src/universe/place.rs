//! Where a place lies in a galaxy: the numbers the density of the stars and the dust there both read.

use bevy::math::DVec3;

use super::galaxy::Shape;
use super::spec::GalaxyKind;

/// Returns 0 below `a`, 1 above `b` and a smooth blend between (also works when `a > b`).
pub(super) fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Where a place lies in a galaxy, worked out once for the stars and the dust there.
#[derive(Debug, Clone, Copy)]
pub(super) struct Place {
    /// The place in radii of the galaxy from its centre.
    pub q: DVec3,
    /// Radii of the galaxy from the axis of the disc.
    pub r: f64,
    /// Radii of the galaxy from its centre.
    pub d: f64,
    /// Share of the density that the void at the centre and the reach of the galaxy leave.
    pub alive: f64,
    /// Angle round the axis of the disc in radians, 0 unless the galaxy is a spiral.
    pub turn: f64,
}

impl Shape {
    /// Returns where the place `p` light-years from the galaxy centre lies in it.
    pub fn place(&self, p: DVec3) -> Place {
        let q = p / self.radius;
        let (r, d) = (q.x.hypot(q.z), q.length());
        let turn = match self.kind {
            GalaxyKind::Spiral => q.z.atan2(q.x),
            _ => 0.0,
        };
        Place {
            q,
            r,
            d,
            alive: smooth(0.01, 0.03, d) * smooth(1.4, 1.0, d),
            turn,
        }
    }
}
