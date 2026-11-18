//! Moons on the system chart: each moon is drawn on a tiny orbit around the mark of its
//! planet, since at the scale of the system it would sit on it.
//!
//! The chart point of a moon is its planet's, plus one step per rank in the direction the
//! moon really lies. The steps only part from the planet mark from [`MOON_ZOOM`] in, so below
//! it the moons are hidden and the planet shows how many it has.

use bevy::math::Vec2;

use super::chart::{Dot, flat};
use super::route::Target;
use super::rows::Line;

/// Zoom from which the moons are drawn apart from their planet.
pub const MOON_ZOOM: f32 = 6.0;

/// Chart units between a planet and its closest moon, and between two moon orbits.
pub const MOON_STEP: f32 = 0.01;

/// Returns true when `target` is a moon.
pub fn is_moon(target: Target) -> bool {
    matches!(target, Target::Body(id) if id.is_moon())
}

/// Returns the row of the planet that the moon of row `index` orbits, when the table has it.
pub fn host(lines: &[Line], index: usize) -> Option<usize> {
    let Target::Body(id) = lines.get(index)?.target else {
        return None;
    };
    let planet = Target::Body(id.planet());
    id.is_moon()
        .then(|| lines.iter().position(|line| line.target == planet))
        .flatten()
}

/// Returns how many moon rows of the table orbit the planet of row `index`.
pub fn count(lines: &[Line], index: usize) -> usize {
    let orbits = |moon: &usize| host(lines, *moon) == Some(index);
    (0..lines.len()).filter(orbits).count()
}

/// Returns true when `dot` is drawn at `zoom`: a moon only once it parts from its planet.
pub fn seen(dot: Dot, lines: &[Line], zoom: f32) -> bool {
    let moon = |index: usize| lines.get(index).is_some_and(|line| is_moon(line.target));
    match dot {
        Dot::Row(index) => zoom >= MOON_ZOOM || !moon(index),
        _ => true,
    }
}

/// Moves the chart point of every moon of `lines` onto its tiny orbit around its planet,
/// toward where the moon really lies. A moon whose planet is not in the table stays put.
pub fn settle(lines: &[Line], points: &mut [Vec2]) {
    for index in 0..lines.len().min(points.len()) {
        let (Some(planet), Target::Body(id)) = (host(lines, index), lines[index].target) else {
            continue;
        };
        let toward = flat(lines[index].at - lines[planet].at).normalize_or(Vec2::X);
        points[index] = points[planet] + toward * MOON_STEP * f32::from(id.moon);
    }
}

/// Returns where the ship is drawn: beside the row nearest to it, as far from that row's
/// chart point as it is from where that row would be without its move onto a moon orbit.
/// `raw` holds the chart points before [`settle`], `points` after.
pub fn ship(lines: &[Line], (raw, points): (&[Vec2], &[Vec2]), at: Vec2) -> Vec2 {
    let nearest = (0..lines.len().min(points.len()))
        .min_by(|a, b| lines[*a].distance.total_cmp(&lines[*b].distance));
    nearest.map_or(at, |index| points[index] + at - raw[index])
}
