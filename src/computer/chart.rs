//! The chart: the rows of a tab laid out as a map seen from above the plane of the system or
//! of the galaxy.
//!
//! A system is drawn around its star with the square root of the distance, so the inner
//! planets do not vanish beside the outer ones. Stars and galaxies are drawn around the ship
//! at their true distances.

use std::f32::consts::TAU;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::board::Tab;
use super::rows::Line;
use crate::space::UniPos;
use crate::universe::PlanetClass;

/// Share of the half width of the map the farthest thing is drawn at.
const FILL: f32 = 0.88;

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
        PlanetClass::Lava => &[[0, 1, 3, 1], [1, 0, 1, 3]],
        PlanetClass::Barren => &[[1, 1, 1, 1]],
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
fn flat(offset: DVec3) -> Vec2 {
    Vec2::new(offset.x as f32, -offset.z as f32)
}

/// Returns the points of the system chart: the star in the middle, each planet on its dotted
/// orbit, and the ship. The first row is the star.
fn system(lines: &[Line], ship: UniPos) -> Vec<(Dot, Vec2)> {
    let Some(origin) = lines.first().map(|star| star.at) else {
        return Vec::new();
    };
    let around = |at: UniPos| flat(at - origin);
    let far = lines.iter().map(|line| around(line.at).length());
    let reach = far.fold(1.0, f32::max);
    let scaled = |at: Vec2| at.normalize_or_zero() * (at.length() / reach).min(1.0).sqrt() * FILL;
    let mut dots = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let at = scaled(around(line.at));
        let ring = (0..ORBIT_DOTS).map(|step| Vec2::from_angle(step as f32 * TAU / 40.0));
        dots.extend(
            ring.filter(|_| index > 0)
                .map(|dir| (Dot::Orbit, dir * at.length())),
        );
        dots.push((Dot::Row(index), at));
    }
    dots.push((Dot::Ship, scaled(around(ship))));
    dots
}

/// Returns the points of a chart of far things around the ship, which stands in the middle.
fn field(lines: &[Line], ship: UniPos) -> Vec<(Dot, Vec2)> {
    let around = |line: &Line| flat(line.at - ship);
    let reach = lines
        .iter()
        .map(|line| around(line).length())
        .fold(1.0, f32::max);
    let place = |(index, line): (usize, &Line)| (Dot::Row(index), around(line) / reach * FILL);
    let mut dots: Vec<(Dot, Vec2)> = lines.iter().enumerate().map(place).collect();
    dots.push((Dot::Ship, Vec2::ZERO));
    dots
}

/// Returns every dot of the chart of `tab` with its place on the map, from -1 to 1 on both
/// axes with north up, for the rows `lines` and a ship at `ship`.
pub fn chart(tab: Tab, lines: &[Line], ship: UniPos) -> Vec<(Dot, Vec2)> {
    match tab {
        Tab::System => system(lines, ship),
        Tab::Galaxy | Tab::Universe => field(lines, ship),
    }
}
