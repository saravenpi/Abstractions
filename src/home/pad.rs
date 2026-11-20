//! The landing pad: a marked deck that follows the ground, so a ship resting on the terrain
//! rests on the pad, with a skirt and four corner lights.

use bevy::prelude::*;

use super::draft::Draft;
use super::glow::Glow;
use super::paint::{DARK, DECK, EDGE, MARK};
use super::soup::Rgb;
use super::tune::{PAD, PAD_HALF, PAD_LIFT, SINK};

/// Cells along each side of the pad: a narrow marked strip at both ends and wide cells between.
pub const CELLS: usize = 10;

/// Width in metres of the marked strip along the edge.
const PANEL: f32 = 0.4;

/// Height in metres of a corner light above the ground, and its distance beyond the corner.
const CORNER: (f32, f32) = (0.45, 0.4);

/// Returns the position of grid line `i` across the pad, from one edge to the other.
fn line(i: usize) -> f32 {
    let inner = PAD_HALF - PANEL;
    match i {
        0 => -PAD_HALF,
        CELLS => PAD_HALF,
        _ => -inner + (i - 1) as f32 * 2.0 * inner / (CELLS - 2) as f32,
    }
}

/// Returns the colour of cell `(i, j)`: dashed markings along the edge and a paler centre.
fn tone(i: usize, j: usize) -> Rgb {
    let rim = |k: usize| k == 0 || k == CELLS - 1;
    let middle = |k: usize| k == CELLS / 2 || k + 1 == CELLS / 2;
    match (rim(i) || rim(j), middle(i) && middle(j)) {
        (true, _) if (i + j).is_multiple_of(2) => EDGE,
        (false, true) => MARK,
        _ => DECK,
    }
}

/// Returns the grid of the pad surface, a little above the ground under every grid point, and
/// records those ground points.
fn surface(draft: &mut Draft) -> Vec<Vec<Vec3>> {
    let point = |draft: &mut Draft, i: usize, j: usize| {
        let at = Vec2::new(line(i), line(j));
        let ground = Vec3::new(at.x, draft.ground(at), at.y);
        draft.feet.push(draft.at(ground));
        ground + Vec3::Y * PAD_LIFT
    };
    (0..=CELLS)
        .map(|i| (0..=CELLS).map(|j| point(draft, i, j)).collect())
        .collect()
}

/// Adds the skirt that closes the gap between the edge of the surface and the ground.
fn skirt(draft: &mut Draft, grid: &[Vec<Vec3>]) {
    let drop = Vec3::Y * (PAD_LIFT + SINK);
    let sides = [
        (Vec3::NEG_Z, [0, 0], [1, 0]),
        (Vec3::Z, [0, CELLS], [1, 0]),
        (Vec3::NEG_X, [0, 0], [0, 1]),
        (Vec3::X, [CELLS, 0], [0, 1]),
    ];
    draft.draw(|soup| {
        for (out, start, step) in sides {
            for k in 0..CELLS {
                let at = |k: usize| grid[start[0] + step[0] * k][start[1] + step[1] * k];
                let (a, b) = (at(k), at(k + 1));
                soup.quad([a, b, b - drop, a - drop], out, DARK);
            }
        }
    });
}

/// Adds the four dim lights on short posts just beyond the corners.
fn corners(draft: &mut Draft) {
    let reach = PAD_HALF + CORNER.1;
    for (x, z) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let at = Vec2::new(x, z) * reach;
        let top = draft.ground(at) + CORNER.0;
        draft.leg(at, top, 0.06, DARK);
        let (lo, hi) = (at - Vec2::splat(0.09), at + Vec2::splat(0.09));
        draft.light(
            Glow::Corner,
            lo.extend(top).xzy(),
            hi.extend(top + 0.12).xzy(),
        );
    }
}

/// Draws the pad at its spot.
pub fn build(draft: &mut Draft) {
    draft.stand(PAD, &[], 0.0);
    let grid = surface(draft);
    draft.draw(|soup| {
        for (i, j) in (0..CELLS).flat_map(|i| (0..CELLS).map(move |j| (i, j))) {
            let cell = [
                grid[i][j],
                grid[i + 1][j],
                grid[i + 1][j + 1],
                grid[i][j + 1],
            ];
            soup.quad(cell, Vec3::Y, tone(i, j));
        }
    });
    skirt(draft, &grid);
    corners(draft);
}
