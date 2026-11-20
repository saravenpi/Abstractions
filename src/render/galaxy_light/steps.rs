//! The steps of a line of sight through the light of a galaxy: one sampling rule for a camera
//! inside a galaxy and for one far outside.
//!
//! The rule, from the camera along the line:
//!
//! 1. The part nearer than [`NEAR`] light-years gets one segment that counts its dust and no
//!    stars, because the stars drawn one by one hold that sphere.
//! 2. Past it, a step is [`GROWTH`] of its distance from the camera, so the structure near the
//!    camera resolves and a camera that moves sees the light change without a jump, until the
//!    step reaches a cap.
//! 3. The cap is [`STEP`] of the radius of the galaxy: the disc and the core have a deviation of
//!    0.02 to 0.1 radii, so no structure falls between two samples. The rest of the line is cut
//!    in equal steps of at most the cap.
//! 4. The cap is raised when the chord is long enough to need more than [`MOST`] steps in all,
//!    and lowered when it would give fewer than [`LEAST`]: a short chord through the slab needs
//!    few, one along the disc is cut at the most.
//!
//! A camera far outside a galaxy meets rule 3 at once: equal steps along the chord. A camera
//! inside meets rule 2 first, about 25 short steps, then rule 3.

use super::march::NEAR;
use crate::universe::LY;

/// Fewest steps along the part of a line of sight that lies in the light of one galaxy.
pub const LEAST: usize = 6;

/// Most steps along that part: the growing steps near the camera, about 25, come on top of
/// the equal ones, so this is the share of the budget the rest may use.
pub const MOST: usize = 72;

/// Share of the radius of the galaxy that a step is at most.
pub const STEP: f64 = 0.03;

/// Share of its distance from the camera that a step is at most near the camera.
pub const GROWTH: f64 = 0.25;

/// Returns the segments, as pairs of distances in metres along the line of sight, that a line of
/// sight in the light of a galaxy of `radius` metres between `from` and `to` is cut into, by the
/// rule of this module.
pub fn segments(from: f64, to: f64, radius: f64) -> Vec<(f64, f64)> {
    let near = f64::from(NEAR) * LY;
    let start = from.max(near);
    let mut cuts = Vec::new();
    if from < near {
        cuts.push((from, near.min(to)));
    }
    if to <= start {
        return cuts;
    }
    let chord = to - start;
    let cap = (STEP * radius)
        .max(chord / MOST as f64)
        .min(chord / LEAST as f64);
    let mut at = start;
    while at < to && GROWTH * at < cap {
        let next = (at * (1.0 + GROWTH)).min(to);
        cuts.push((at, next));
        at = next;
    }
    if at < to {
        let equal = ((to - at) / cap).ceil();
        let step = (to - at) / equal;
        cuts.extend((0..equal as usize).map(|k| (at + step * k as f64, at + step * (k + 1) as f64)));
    }
    cuts
}
