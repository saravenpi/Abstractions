//! Tests that two neighbouring coarse nodes of different levels and cell counts leave no open
//! crack: wherever the edge of one stands above the edge of the other, the skirt of the higher
//! hangs deeper than the gap.

use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;

use super::node::{cells_of, skirt_depth};
use super::span::Span;
use super::style::Style;
use crate::universe::Universe;
use crate::world::globe::{ChunkId, Face};
use crate::world::{Globe, Terrain};

/// Returns the heights of the vertices of node `id` as its mesh draws them, and its cells.
fn heights(terrain: &Terrain, id: ChunkId) -> (Vec<Vec<f32>>, usize) {
    let n = cells_of(terrain, id);
    let across = (1u64 << id.level) as f64 * n as f64;
    let spaced = terrain.spaced((terrain.globe().radius() * FRAC_PI_2 / across) as f32);
    let floor = Style::coarse(1.0, terrain.globe().lands()).floor;
    let rows = Span::node(&spaced, id, n).heights();
    let drawn = rows
        .into_iter()
        .map(|row| row.into_iter().map(|h| h.max(floor)).collect());
    (drawn.collect(), n)
}

/// Returns the deepest gap along the left edge of the fine node `fine` against the right edge of
/// its coarser neighbour one level up that the skirts leave open: positive means a crack.
fn open_gap(terrain: &Terrain, fine: ChunkId) -> f32 {
    let coarse = ChunkId {
        level: fine.level - 1,
        x: fine.x / 2 - 1,
        y: fine.y / 2,
        ..fine
    };
    let ((near, n_fine), (far, n_coarse)) = (heights(terrain, fine), heights(terrain, coarse));
    let relief = |rows: &[Vec<f32>]| {
        let all = rows.iter().flatten();
        all.clone().fold(f32::MIN, |a, b| a.max(*b)) - all.fold(f32::MAX, |a, b| a.min(*b))
    };
    let skirts = (
        skirt_depth(terrain, (fine, n_fine), relief(&near)),
        skirt_depth(terrain, (coarse, n_coarse), relief(&far)),
    );
    let half = f32::from(u8::from(fine.y % 2 == 1)) * 0.5;
    (0..=n_fine)
        .map(|j| {
            let at = (half + j as f32 / n_fine as f32 * 0.5) * n_coarse as f32;
            let k = (at.floor() as usize).min(n_coarse - 1);
            let line =
                far[n_coarse][k] + (far[n_coarse][k + 1] - far[n_coarse][k]) * (at - k as f32);
            let gap = near[0][j] - line;
            match gap >= 0.0 {
                true => gap - skirts.0,
                false => -gap - skirts.1,
            }
        })
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Along every edge between a node and its coarser neighbour, at the normal step of cells and at
/// both steps of the far nodes, the skirt of the higher side closes the gap.
#[test]
fn neighbours_of_every_cell_count_leave_no_crack() {
    let universe = Universe::new(7);
    let globe = Arc::new(Globe::of(&universe.planet(universe.home())));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let leaf = globe.leaf_level();
    let mut checked = 0;
    for above in [1u8, 3, 4, 6, 7] {
        let Some(level) = leaf.checked_sub(above).filter(|level| *level >= 2) else {
            continue;
        };
        let side = 1u32 << level;
        for (x, y) in [(2, 0), (side / 2, side / 2), (side - 2, side / 2 + 1)] {
            let fine = ChunkId {
                face: Face::PosY,
                level,
                x,
                y,
            };
            let open = open_gap(&terrain, fine);
            assert!(open <= 0.0, "{fine:?}: a crack {open} m deep");
            checked += 1;
        }
    }
    assert!(checked >= 6, "{checked} edges checked");
}
