//! What to paint next and how much of the map the patch shown covers. A new patch is centred a
//! little ahead of where the player goes, so a fast flight reaches its middle as it is painted,
//! and the map never shows more than the patch on screen holds.

use std::f32::consts::SQRT_2;

use bevy::math::DVec3;

use super::patch::{DRIFT, Patch};

/// Seconds ahead of the player the middle of a new patch is put: about the time a worker
/// takes to paint the new edge of a patch while the world streams around a fast flight.
const LEAD: f64 = 0.5;

/// Returns the patch to paint for a player in the unit direction `dir` of a planet of `radius`
/// metres and a map of `range` metres, or nothing while the patch `shown` still serves.
pub fn next(shown: Option<Patch>, dir: DVec3, radius: f64, range: f32) -> Option<Patch> {
    let Some(patch) = shown.filter(|patch| patch.range == range) else {
        return Some(Patch::around(dir, radius, range));
    };
    let at = patch.locate(dir);
    patch.stale(at, range).then(|| patch.moved(at))
}

/// Returns the unit direction a player `at` metres from the centre of a planet, moving at
/// `velocity` metres per second, reaches in [`LEAD`] seconds. The step is held to the drift of
/// a patch of `range` metres, so a new patch always holds the player.
pub fn ahead(at: DVec3, velocity: DVec3, range: f32) -> DVec3 {
    let step = velocity * LEAD;
    let most = f64::from(DRIFT * range);
    let held = match step.length() > most {
        true => step * (most / step.length()),
        false => step,
    };
    (at + held).normalize()
}

/// Returns the largest range in metres a map may show through `patch` for a player in unit
/// direction `dir`. The view turns with the heading, so its corners reach the square root of
/// two ranges from the player, and every one of them must stay on the patch.
pub fn cover(patch: &Patch, dir: DVec3) -> f32 {
    let at = patch.locate(dir);
    (patch.half() - at.abs().max_element()) / SQRT_2
}
