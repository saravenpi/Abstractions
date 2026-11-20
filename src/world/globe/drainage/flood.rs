//! The priority flood: every land cell gets the neighbour its water leaves by, pits filled, and the area
//! that drains through it.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::grid::Grid;

/// Where the water of each cell goes, and how much comes through it.
pub struct Flood {
    /// The cell each one drains into, itself where it is a sea cell or the bottom of a closed basin.
    pub next: Vec<u32>,
    /// Land cells draining through each cell, itself included.
    pub area: Vec<u32>,
}

/// Returns `height` as an integer that sorts the same way.
fn key(height: f32) -> u32 {
    let bits = height.to_bits();
    match bits >> 31 {
        1 => !bits,
        _ => bits | 1 << 31,
    }
}

/// Floods the grid `grid` of `height`, from every cell at or under `sea`, or from the lowest when none is,
/// always spreading from the lowest water level: each cell drains into the cell it was reached from, so
/// every path ends in the sea or a basin, and a pit is filled to its lowest rim.
pub fn flood(grid: Grid, height: &[f32], sea: f32) -> Flood {
    let mut next: Vec<u32> = (0..height.len() as u32).collect();
    let mut seen: Vec<bool> = height.iter().map(|h| *h <= sea).collect();
    let mut level = height.to_vec();
    let mut order = Vec::with_capacity(height.len());
    let mut heap = BinaryHeap::new();
    if !seen.contains(&true) {
        let low = (0..height.len()).min_by(|a, b| height[*a].total_cmp(&height[*b]));
        low.into_iter().for_each(|k| seen[k] = true);
    }
    (0..height.len())
        .filter(|k| seen[*k])
        .for_each(|k| heap.push(Reverse((key(level[k]), k))));
    while let Some(Reverse((_, k))) = heap.pop() {
        order.push(k);
        for to in grid.around(k) {
            if std::mem::replace(&mut seen[to], true) {
                continue;
            }
            next[to] = k as u32;
            level[to] = level[k].max(height[to]);
            heap.push(Reverse((key(level[to]), to)));
        }
    }
    let mut area: Vec<u32> = height.iter().map(|h| u32::from(*h > sea)).collect();
    for &k in order.iter().rev().filter(|k| next[**k] != **k as u32) {
        area[next[k] as usize] += area[k];
    }
    Flood { next, area }
}
