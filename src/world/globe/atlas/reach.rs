//! Cells in reach: every atlas cell near enough to another by true distance on the sphere, across face edges
//! and cube corners.
//!
//! The squares of a gnomonic chart shrink towards the edges of a face and stretch past them, so a walk of
//! so many squares covers fewer metres there. The cells are spread from neighbour to neighbour instead, and
//! each is kept when any point of it may lie within the reach of any point of the first.

use bevy::math::DVec3;

use super::cell::{Cell, STEPS};

/// How far past a side of a cell, in cells, the point lies that names the cell touching it.
const PAST: f64 = 0.01;

/// Returns the unit direction of the middle of `cell` on a planet of `n` cells per face side, and the angle
/// from it to its farthest corner.
fn span(cell: Cell, n: i32) -> (DVec3, f64) {
    let side = STEPS as f64;
    let middle = cell.dir([side * 0.5; 2], n);
    let corners = [[0.0, 0.0], [side, 0.0], [0.0, side], [side, side]];
    let wide = corners.map(|c| cell.dir(c, n).angle_between(middle));
    (middle, wide.into_iter().fold(0.0, f64::max))
}

/// Returns the cells touching `cell` along its sides and at its corners, on its face or the next one.
fn touching(cell: Cell, n: i32) -> impl Iterator<Item = Cell> {
    let steps = [-PAST, 0.5, 1.0 + PAST].map(|k| k * STEPS as f64);
    let places = steps.into_iter().flat_map(move |a| steps.map(|b| [a, b]));
    places
        .map(move |local| Cell::locate(cell.dir(local, n), n).0)
        .filter(move |found| *found != cell)
}

/// Returns `cell` and every cell of a planet of `n` cells per face side any point of which may lie within
/// the angle `reach`, in radians, of any point of `cell`: those whose middle is within the reach plus the
/// half diagonals of both.
pub fn around(cell: Cell, n: i32, reach: f64) -> Vec<Cell> {
    let (middle, own) = span(cell, n);
    let mut cells = vec![cell];
    let mut next = 0;
    while next < cells.len() {
        for found in touching(cells[next], n) {
            let (at, wide) = span(found, n);
            let near = at.angle_between(middle) <= reach + own + wide;
            if near && !cells.contains(&found) {
                cells.push(found);
            }
        }
        next += 1;
    }
    cells
}
