//! Flow maps: for one atlas cell, every channel segment that crosses it, sorted into 32 by 32 bins.
//!
//! A ground sample reads only the few segments of its bin, so a cell crossed by no flow costs one empty
//! check. A flow ends within [`LONGEST`] steps of its spring, so the springs of the cells that close are
//! enough.

use bevy::math::DVec3;

use super::cell::{Cell, STEPS};
use super::channel::pull;
use super::pits::{around, local_of};
use super::trace::{LONGEST, STEP};
use crate::world::globe::Globe;

/// Bins along each side of a cell.
const BINS: usize = 32;

/// Metres between two points of a segment sorted into the bins.
const STRIDE: f64 = 15.0;

/// One straight piece of a channel between two flow points, in metres from the planet centre.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Segment {
    /// Upstream end.
    from: DVec3,
    /// Downstream end.
    to: DVec3,
    /// Bed heights at both ends.
    bed: [f32; 2],
    /// Half widths at both ends.
    half: [f32; 2],
}

/// The channel segments crossing one cell, binned.
pub struct FlowMap {
    /// Every segment near the cell.
    segments: Vec<Segment>,
    /// Where the list of each bin starts in `items`, and one more entry for the end.
    starts: Vec<u32>,
    /// The segments of every bin, one bin after the other.
    items: Vec<u32>,
}

impl FlowMap {
    /// Builds the map of `cell` from the flows springing close enough to reach it.
    pub fn build(globe: &Globe, cell: Cell) -> Self {
        let segments = gather(globe, cell);
        let mut bins = vec![Vec::new(); BINS * BINS];
        let n = globe.cells();
        for (index, s) in segments.iter().enumerate() {
            let points = ((s.to - s.from).length() / STRIDE).ceil().max(1.0) as usize;
            for k in 0..=points {
                let dir = s.from.lerp(s.to, k as f64 / points as f64).normalize();
                if let Some(local) = local_of(cell, dir, n) {
                    mark(&mut bins, local, index as u32);
                }
            }
        }
        let mut starts = vec![0];
        bins.iter()
            .for_each(|b| starts.push(starts[starts.len() - 1] + b.len() as u32));
        let items = bins.concat();
        Self {
            segments,
            starts,
            items,
        }
    }

    /// Returns the ground `height` at planet point `p`, `local` grid steps into the cell, with the channels
    /// of its bin cut in.
    pub fn carve(&self, p: DVec3, local: [f64; 2], height: f32) -> f32 {
        self.near(p, local).fold(height, |h, (distance, s, t)| {
            let (half, bed) = (along(s.half, t), along(s.bed, t));
            let weight = pull(distance, half);
            match weight > 0.0 && bed < height {
                true => h.min(height + (bed - height) * weight),
                false => h,
            }
        })
    }

    /// Returns true when planet point `p`, `local` grid steps into the cell, lies in a channel bed, between
    /// its banks.
    pub fn bed(&self, p: DVec3, local: [f64; 2]) -> bool {
        self.near(p, local)
            .any(|(distance, s, t)| distance < along(s.half, t))
    }

    /// Returns the segments of the bin holding `local`, each with the distance from `p` to it and the share
    /// of the way along it of the nearest point.
    fn near(&self, p: DVec3, local: [f64; 2]) -> impl Iterator<Item = (f32, &Segment, f32)> {
        let bin = local.map(|c| ((c * (BINS / STEPS) as f64) as usize).min(BINS - 1));
        let k = bin[0] * BINS + bin[1];
        let range = match self.items.is_empty() {
            true => 0..0,
            false => self.starts[k] as usize..self.starts[k + 1] as usize,
        };
        self.items[range].iter().map(move |&s| {
            let s = &self.segments[s as usize];
            let along = s.to - s.from;
            let t = ((p - s.from).dot(along) / along.length_squared()).clamp(0.0, 1.0);
            ((p - (s.from + along * t)).length() as f32, s, t as f32)
        })
    }
}

/// Returns the value `t` of the way from the first to the second of `ends`.
fn along(ends: [f32; 2], t: f32) -> f32 {
    ends[0] + (ends[1] - ends[0]) * t
}

/// Adds segment `index` to the bin at `local` grid steps and to the bins around it.
fn mark(bins: &mut [Vec<u32>], local: [f64; 2], index: u32) {
    let at = local.map(|c| (c * (BINS / STEPS) as f64) as i64);
    for (di, dj) in (-1..=1).flat_map(|i| (-1..=1).map(move |j| (i, j))) {
        let (i, j) = (at[0] + di, at[1] + dj);
        if (0..BINS as i64).contains(&i) && (0..BINS as i64).contains(&j) {
            let bin = &mut bins[i as usize * BINS + j as usize];
            if bin.last() != Some(&index) {
                bin.push(index);
            }
        }
    }
}

/// Returns every channel segment of the flows springing in the cells around `cell`.
fn gather(globe: &Globe, cell: Cell) -> Vec<Segment> {
    let n = globe.cells();
    let [wide, tall] = cell.size(n, globe.radius());
    let reach = 2 * (LONGEST as f64 * STEP / wide.min(tall)).ceil() as i32 + 2;
    let cells = around(cell, n, reach);
    let radius = globe.radius();
    let flows = cells
        .iter()
        .flat_map(|&c| globe.sources(c).as_ref().clone());
    let pairs = flows.flat_map(|f| f.nodes.windows(2).map(|w| (w[0], w[1])).collect::<Vec<_>>());
    pairs
        .map(|(a, b)| Segment {
            from: a.dir * radius,
            to: b.dir * radius,
            bed: [a.bed, b.bed],
            half: [a.half, b.half],
        })
        .collect()
}
