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

/// Returns which cells of `height` the flood starts from: those at or under `sea`, or the lowest when none is.
fn wet(height: &[f32], sea: f32) -> Vec<bool> {
    let mut seen: Vec<bool> = height.iter().map(|h| *h <= sea).collect();
    if !seen.contains(&true) {
        let low = (0..height.len()).min_by(|a, b| height[*a].total_cmp(&height[*b]));
        low.into_iter().for_each(|k| seen[k] = true);
    }
    seen
}

/// Floods the grid `grid` of `height`, from every cell at or under `sea`, or from the lowest when none is,
/// always spreading from the lowest water level: each cell drains into the cell it was reached from, so
/// every path ends in the sea or a basin, and a pit is filled to its lowest rim.
pub fn flood(grid: Grid, height: &[f32], sea: f32) -> Flood {
    let mut next: Vec<u32> = (0..height.len() as u32).collect();
    let mut seen = wet(height, sea);
    let mut level = height.to_vec();
    let mut order = Vec::with_capacity(height.len());
    let packed = |level: f32, k: usize| Reverse((u64::from(key(level)) << 32) | k as u64);
    let seeds: Vec<_> = (0..height.len())
        .filter(|k| seen[*k])
        .map(|k| packed(level[k], k))
        .collect();
    let mut heap = BinaryHeap::from(seeds);
    while let Some(Reverse(top)) = heap.pop() {
        let k = (top & u64::from(u32::MAX)) as usize;
        order.push(k);
        for to in grid.around(k) {
            if std::mem::replace(&mut seen[to], true) {
                continue;
            }
            next[to] = k as u32;
            level[to] = level[k].max(height[to]);
            heap.push(packed(level[to], to));
        }
    }
    let mut area: Vec<u32> = height.iter().map(|h| u32::from(*h > sea)).collect();
    for &k in order.iter().rev().filter(|k| next[**k] != **k as u32) {
        area[next[k] as usize] += area[k];
    }
    Flood { next, area }
}
