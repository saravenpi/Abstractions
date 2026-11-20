//! The scale of the system chart: the star it is centred on, how far its farthest row reaches,
//! and where the ship is drawn without drawing the whole chart again.

use bevy::math::Vec2;

use super::chart::{Dot, FILL, flat};
use super::route::Target;
use super::rows::Line;
use crate::space::UniPos;

/// Returns the point the system chart is centred on, and how far in metres its farthest row
/// lies from it. The point is the star, or the first row when the filter left the star out.
pub(super) fn reading(lines: &[Line]) -> Option<(UniPos, f32)> {
    let star = lines
        .iter()
        .find(|line| matches!(line.target, Target::Star(_)));
    let origin = star.or(lines.first())?.at;
    let far = lines.iter().map(|line| flat(line.at - origin).length());
    Some((origin, far.fold(1.0, f32::max)))
}

/// Returns the chart point of `offset`, metres east and north of the centre, on a chart that
/// reaches `reach` metres: the square root of the distance, so inner planets stay in sight.
pub(super) fn squash(offset: Vec2, reach: f32) -> Vec2 {
    offset.normalize_or_zero() * (offset.length() / reach).min(1.0).sqrt() * FILL
}

/// Returns where the ship at `ship` is drawn on the system chart of `lines` whose dots are
/// `dots`, the way a full chart draws it: beside the row nearest to it, as far from that row
/// as it is in space. It reads the rows and the dots as they are and allocates nothing, so a
/// moving ship can be placed every frame.
pub fn ship_dot(lines: &[Line], dots: &[(Dot, Vec2)], ship: UniPos) -> Vec2 {
    let Some((origin, reach)) = reading(lines) else {
        return Vec2::ZERO;
    };
    let scaled = |at: UniPos| squash(flat(at - origin), reach);
    let nearest = (0..lines.len()).min_by(|a, b| lines[*a].distance.total_cmp(&lines[*b].distance));
    let drawn = |index: usize| {
        let found = dots.iter().find(|(dot, _)| *dot == Dot::Row(index));
        found.map(|(_, at)| *at + scaled(ship) - scaled(lines[index].at))
    };
    nearest.and_then(drawn).unwrap_or_else(|| scaled(ship))
}
