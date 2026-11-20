//! The steps of a line of sight through a galaxy: short where the camera is, so the structure near
//! it resolves, and growing by a constant ratio out to the far edge, where a galaxy 50,000
//! light-years across needs no finer. The same number of steps as a plain division, so the
//! bake costs what it did.

use super::march::NEAR;
use crate::universe::LY;

/// Steps along the part of a line of sight that lies in the light of one galaxy, past [`NEAR`].
pub const STEPS: usize = 24;

/// Returns the segments, as pairs of distances in metres along the line of sight, that a line
/// of sight in the light of a galaxy between `from` and `to` is cut into.
///
/// A line that starts nearer than [`NEAR`] light-years has first one segment up to it: its dust
/// counts, and no star gives light there, because the stars drawn one by one hold that sphere.
/// Then [`STEPS`] segments, each longer than the last by the ratio `(to / start)^(1 / STEPS)`,
/// from `start`, [`NEAR`] or `from` if that is farther, to `to`.
pub fn segments(from: f64, to: f64) -> impl Iterator<Item = (f64, f64)> {
    let near = f64::from(NEAR) * LY;
    let start = from.max(near);
    let ratio = (to / start).max(1.0).powf(1.0 / STEPS as f64);
    let lead = (from < near).then_some((from, near.min(to)));
    let grown = (0..STEPS).filter(move |_| to > start).map(move |k| {
        (
            start * ratio.powi(k as i32),
            start * ratio.powi(k as i32 + 1),
        )
    });
    lead.into_iter().chain(grown)
}
