//! The chart: the rows of a tab laid out as a map seen from above the plane of the system or
//! of the galaxy.
//!
//! A system is drawn around its star with the logarithm of the distance, see [`scale`], so the
//! inner planets do not vanish beside the outer ones. Stars and galaxies are drawn around the
//! ship at their true distances: a neighbourhood of stars a few light-years apart fills the
//! map, and a far cluster or a far home leaves it for the pin of the rim, see
//! [`super::rim::spread`].

use std::f32::consts::TAU;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::lens::Lens;
use super::moons;
use super::route::Target;
use super::rows::Line;
use super::scale;
use super::tab::Tab;
use crate::space::UniPos;
use crate::universe::PlanetClass;

/// Share of the half width of the map the farthest thing is drawn at.
pub const FILL: f32 = 0.88;

/// Dots that trace one orbit.
const ORBIT_DOTS: usize = 40;

/// Cells along one side of the grid a class mark is drawn on.
pub const GRID: u8 = 3;

/// Returns the mark of a class on the map: filled blocks of the grid, each given as the
/// column and the row of its corner, then its width and its height in cells.
///
/// An island world is a full square, an ocean world a ring, a desert a flat bar, an ice world
/// five flakes, a lava world a cross and a barren world a speck.
pub const fn mark(class: PlanetClass) -> &'static [[u8; 4]] {
    match class {
        PlanetClass::Island => &[[0, 0, 3, 3]],
        PlanetClass::Ocean => &[[0, 0, 3, 1], [0, 2, 3, 1], [0, 1, 1, 1], [2, 1, 1, 1]],
        PlanetClass::Desert => &[[0, 1, 3, 1]],
        PlanetClass::Ice => &[
            [0, 0, 1, 1],
            [2, 0, 1, 1],
            [1, 1, 1, 1],
            [0, 2, 1, 1],
            [2, 2, 1, 1],
        ],
        PlanetClass::Lava => &[[0, 1, 3, 1], [1, 0, 1, 1], [1, 2, 1, 1]],
        PlanetClass::Barren => &[[1, 1, 1, 1]],
        PlanetClass::GasGiant => &[[0, 0, 3, 1], [0, 2, 3, 1]],
    }
}

/// What a dot of the chart stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dot {
    /// The row with this index: a star, a planet or a galaxy.
    Row(usize),
    /// The ship.
    Ship,
    /// One point of the orbit of a planet.
    Orbit,
}

/// Returns `offset` seen from above: east to the right and north up, in metres.
pub(super) fn flat(offset: DVec3) -> Vec2 {
    Vec2::new(offset.x as f32, -offset.z as f32)
}

/// Returns the points of the system chart: the star in the middle, each planet on its dotted
/// orbit with its moons on tiny orbits around it, and the ship. The star is the row of a star,
/// or the first row when the filter left it out.
fn system(lines: &[Line], ship: UniPos) -> Vec<(Dot, Vec2)> {
    let Some((origin, radial)) = scale::reading(lines) else {
        return Vec::new();
    };
    let around = |at: UniPos| flat(at - origin);
    let scaled = |at: Vec2| scale::squash(at, radial);
    let raw: Vec<Vec2> = lines.iter().map(|line| scaled(around(line.at))).collect();
    let mut points = raw.clone();
    moons::settle(lines, &mut points);
    let mut dots = Vec::new();
    for (index, (line, at)) in lines.iter().zip(&points).enumerate() {
        let ring = (0..ORBIT_DOTS).map(|step| Vec2::from_angle(step as f32 * TAU / 40.0));
        let circles = matches!(line.target, Target::Body(_)) && !moons::is_moon(line.target);
        dots.extend(
            ring.filter(|_| circles)
                .map(|dir| (Dot::Orbit, dir * at.length())),
        );
        dots.push((Dot::Row(index), *at));
    }
    let ship = moons::ship(lines, (&raw, &points), scaled(around(ship)));
    dots.push((Dot::Ship, ship));
    dots
}

/// Returns the farthest horizontal distance in metres from `ship` to a row, at least one metre.
/// It is worked out in double precision: a squared distance between galaxies overflows a float.
fn reach(lines: &[Line], ship: UniPos) -> f64 {
    let gap = |line: &Line| {
        let offset = line.at - ship;
        offset.x.hypot(offset.z)
    };
    lines.iter().map(gap).fold(1.0, f64::max)
}

/// Returns the points of a chart of far things around the ship, which stands in the middle.
fn field(lines: &[Line], ship: UniPos) -> Vec<(Dot, Vec2)> {
    let reach = reach(lines, ship);
    let place = |(index, line): (usize, &Line)| {
        let offset = (line.at - ship) / reach * f64::from(FILL);
        (Dot::Row(index), flat(offset))
    };
    let mut dots: Vec<(Dot, Vec2)> = lines.iter().enumerate().map(place).collect();
    dots.push((Dot::Ship, Vec2::ZERO));
    dots
}

/// Returns the mark under the pointer at `pixel`: the row or the ship drawn nearest to it,
/// within `radius` pixels. The dots of the orbits are not marks.
pub fn hit<'a>(
    dots: impl IntoIterator<Item = &'a (Dot, Vec2)>,
    lens: Lens,
    pixel: Vec2,
    radius: f32,
) -> Option<Dot> {
    let marks = dots.into_iter().filter(|(dot, _)| *dot != Dot::Orbit);
    let near = marks.map(|(dot, at)| (*dot, lens.to_map(*at).distance(pixel)));
    let inside = near.filter(|(_, gap)| *gap <= radius);
    inside
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(dot, _)| dot)
}

/// Returns the metres one chart unit stands for on the chart of `tab`, or nothing for a system
/// chart, whose orbits are drawn closer together than they are.
pub fn span(tab: Tab, lines: &[Line], ship: UniPos) -> Option<f64> {
    (tab != Tab::System).then_some(reach(lines, ship) / f64::from(FILL))
}

/// Returns every dot of the chart of `tab` with its place on the map, from -1 to 1 on both
/// axes with north up, for the rows `lines` and a ship at `ship`.
pub fn chart(tab: Tab, lines: &[Line], ship: UniPos) -> Vec<(Dot, Vec2)> {
    match tab {
        Tab::System => system(lines, ship),
        Tab::Galaxy | Tab::Universe => field(lines, ship),
    }
}
