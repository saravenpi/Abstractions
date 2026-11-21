//! Paths: the way a herd walks from one waypoint to the next, straight where the ground is not too
//! steep for its body, else through the centre of its home range, whose straight lines to every
//! waypoint are walkable by how waypoints are drawn.

use bevy::math::DVec3;

use super::sphere::{arc, bearing, slerp};

/// Metres between two height samples along a path: under the width of the smallest pond, so a herd
/// never walks through one between two samples.
const STEP: f64 = 12.0;

/// Most height samples along one path: a path of up to 768 m is read every [`STEP`].
const SAMPLES: usize = 64;

/// Returns true when the ground along the great circle from `a` to `b`, on a planet of `radius`
/// metres, read by `height`, never rises or falls more steeply than `slope` radians between two
/// samples, nor dips under `floor` metres, the deepest water the body wades.
pub(in crate::life) fn walkable(
    height: &dyn Fn(DVec3) -> f32,
    (a, b): (DVec3, DVec3),
    radius: f64,
    (slope, floor): (f32, f32),
) -> bool {
    if slope >= std::f32::consts::FRAC_PI_2 && floor == f32::NEG_INFINITY {
        return true;
    }
    let grade = f64::from(slope).tan();
    let span = arc(a, b) * radius;
    let n = ((span / STEP).ceil() as usize).clamp(1, SAMPLES);
    let step = span / n as f64;
    let mut last = height(a);
    if last < floor {
        return false;
    }
    (1..=n).all(|i| {
        let next = height(slerp(a, b, i as f64 / n as f64));
        let fine = f64::from((next - last).abs()) <= grade * step && next >= floor;
        last = next;
        fine
    })
}

/// The way from one waypoint to the next, through `via` when the straight way is too steep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::life) struct Path {
    /// Where it starts.
    pub from: DVec3,
    /// Where it turns, if it does.
    pub via: Option<DVec3>,
    /// Where it ends.
    pub to: DVec3,
}

impl Path {
    /// Returns the metres of its legs on a planet of `radius` metres: to the turn, and after it.
    fn legs(&self, radius: f64) -> (f64, f64) {
        match self.via {
            Some(via) => (arc(self.from, via) * radius, arc(via, self.to) * radius),
            None => (arc(self.from, self.to) * radius, 0.0),
        }
    }

    /// Returns its length in metres on a planet of `radius` metres.
    pub fn length(&self, radius: f64) -> f64 {
        let (a, b) = self.legs(radius);
        a + b
    }

    /// Returns the unit direction `s` metres along it on a planet of `radius` metres, and the unit
    /// tangent it heads along there.
    pub fn at(&self, s: f64, radius: f64) -> (DVec3, DVec3) {
        let (a, b) = self.legs(radius);
        let (start, end, u) = match self.via {
            Some(via) if s < a => (self.from, via, s / a.max(1e-9)),
            Some(via) => (via, self.to, (s - a) / b.max(1e-9)),
            None => (self.from, self.to, s / a.max(1e-9)),
        };
        let centre = slerp(start, end, u.clamp(0.0, 1.0));
        (centre, bearing(centre, start, end))
    }
}
