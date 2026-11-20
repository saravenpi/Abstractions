//! The scale of the system chart: the star it is centred on, the logarithmic way a distance
//! becomes a radius, and where the ship is drawn without drawing the whole chart again.
//!
//! The planets of a system lie from 0.03 to 40 AU, and the generator spaces their orbits evenly
//! in the logarithm of the distance. A linear chart draws the inner ones on the star, and the
//! square root of the distance, which the chart used before, still crowds them within three
//! pixels of it. A logarithmic radius gives each orbit about the same room on the map.

use bevy::math::Vec2;

use super::chart::{Dot, FILL, flat};
use super::route::Target;
use super::rows::Line;
use crate::space::UniPos;

/// How a system chart turns a distance from its star into a radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Radial {
    /// Metres of the nearest row from the centre: the distance that gets the first step.
    pub(super) unit: f32,
    /// Metres of the farthest row, drawn at the edge of the chart.
    pub(super) reach: f32,
}

impl Radial {
    /// Returns the share of the chart radius, from 0 to 1, that `metres` from the star is drawn
    /// at: `ln(1 + d / unit)` over the same for the reach. The star is at 0 and the farthest
    /// row at 1. Stays in view for any distance, from a moon's orbit to another star.
    pub fn share(self, metres: f32) -> f32 {
        let log = |metres: f32| (1.0 + metres / self.unit).ln();
        (log(metres) / log(self.reach)).clamp(0.0, 1.0)
    }
}

/// Returns the point the system chart is centred on and its radial scale. The point is the
/// star, or the first row when the filter left the star out. The unit is the distance of the
/// nearest row from the point, so the innermost orbit of any system, a red dwarf's included,
/// takes the same room, and the reach is the distance of the farthest.
pub(super) fn reading(lines: &[Line]) -> Option<(UniPos, Radial)> {
    let star = lines
        .iter()
        .find(|line| matches!(line.target, Target::Star(_)));
    let origin = star.or(lines.first())?.at;
    let gaps = || lines.iter().map(|line| flat(line.at - origin).length());
    let reach = gaps().fold(1.0, f32::max);
    let unit = gaps().filter(|gap| *gap > 0.0).fold(reach, f32::min);
    Some((origin, Radial { unit, reach }))
}

/// Returns the chart point of `offset`, metres east and north of the centre, on a chart with
/// the radial scale `radial`.
pub(super) fn squash(offset: Vec2, radial: Radial) -> Vec2 {
    offset.normalize_or_zero() * radial.share(offset.length()) * FILL
}

/// Returns the chart point of the place `at` on the system chart of `lines`, or nothing for a
/// chart with no row. Markers that stand on no row are placed with it.
pub fn system_point(lines: &[Line], at: UniPos) -> Option<Vec2> {
    let (origin, radial) = reading(lines)?;
    Some(squash(flat(at - origin), radial))
}

/// Returns where the ship at `ship` is drawn on the system chart of `lines` whose dots are
/// `dots`, the way a full chart draws it: beside the row nearest to it, as far from that row
/// as it is in space. It reads the rows and the dots as they are and allocates nothing, so a
/// moving ship can be placed every frame.
pub fn ship_dot(lines: &[Line], dots: &[(Dot, Vec2)], ship: UniPos) -> Vec2 {
    let Some((origin, radial)) = reading(lines) else {
        return Vec2::ZERO;
    };
    let scaled = |at: UniPos| squash(flat(at - origin), radial);
    let nearest = (0..lines.len()).min_by(|a, b| lines[*a].distance.total_cmp(&lines[*b].distance));
    let drawn = |index: usize| {
        let found = dots.iter().find(|(dot, _)| *dot == Dot::Row(index));
        found.map(|(_, at)| *at + scaled(ship) - scaled(lines[index].at))
    };
    nearest.and_then(drawn).unwrap_or_else(|| scaled(ship))
}
