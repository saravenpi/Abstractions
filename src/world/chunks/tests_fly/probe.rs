//! The probe of a flyover: what popped into view this frame, how near the unsettled ground is and
//! how long the oldest request has waited.

use std::collections::{HashMap, HashSet};

use super::super::Chunks;
use super::gaps::{Eye, Gap, gaps};
use crate::world::globe::ChunkId;

/// The kinds of things that appear: a coarse node, a detail chunk, a terrain level, ground cover.
pub const KINDS: usize = 4;

/// Something drawn: its kind, its terrain level when that is what changed, and its place.
type Mark = (usize, u8, ChunkId);

/// What one frame of a flyover showed.
#[derive(Debug, Clone, Copy, Default)]
pub struct Sample {
    /// Distance to the nearest unsettled ground ahead, in metres.
    pub ground: f64,
    /// Distance to the nearest unsettled ground in any direction.
    pub around: f64,
    /// Distance to the nearest ground ahead that only lacks its fine detail.
    pub fine: f64,
    /// Things that appeared within the fog, by kind.
    pub pops: [u32; KINDS],
    /// Of those, the ones that fade in instead of appearing at once.
    pub faded: u32,
    /// Requests that wait: unsettled nodes and leaves.
    pub queue: usize,
    /// Frames the oldest of them has waited.
    pub age: u32,
}

/// Follows what is drawn from frame to frame.
#[derive(Default)]
pub struct Probe {
    seen: Option<HashSet<Mark>>,
    since: HashMap<ChunkId, u32>,
    frame: u32,
}

/// Returns everything `chunks` draws now.
fn marks(chunks: &Chunks) -> HashSet<Mark> {
    let nodes = chunks.nodes.live.iter().filter(|(_, node)| node.visible);
    let mut all: HashSet<Mark> = nodes.map(|(id, _)| (0, 0, *id)).collect();
    for (id, l) in chunks.loaded.iter().filter(|(_, l)| l.shown) {
        all.insert((1, 0, *id));
        all.insert((2, l.detail.lod as u8, *id));
        if l.detail.turf.drawn() {
            all.insert((3, 0, *id));
        }
    }
    all
}

/// Returns true when the thing of `kind` that appeared at `id` fades in.
fn fading(chunks: &Chunks, kind: usize, id: ChunkId) -> bool {
    let node = chunks.nodes.live.get(&id).map(|node| node.entity);
    let chunk = chunks.loaded.get(&id);
    let entity = match kind {
        0 => node,
        1 | 2 => chunk.map(|l| l.entity),
        3 => chunk.and_then(|l| l.detail.turf.grown().flatten()),
        _ => None,
    };
    entity.is_some_and(|entity| chunks.fades.way(entity).is_some())
}

/// Returns the least distance from the eye to the gaps accepted by `keep`.
fn nearest(eye: &Eye, gaps: &[Gap], keep: impl Fn(&Gap) -> bool) -> f64 {
    let kept = gaps.iter().filter(|gap| keep(gap));
    kept.map(|gap| eye.distance(gap.id))
        .fold(f64::INFINITY, f64::min)
}

impl Probe {
    /// Counts what appeared since the last frame within `fog` metres of the eye. A level or a
    /// cover only counts on a chunk that was already drawn.
    fn pops(&mut self, chunks: &Chunks, eye: &Eye, sample: &mut Sample) {
        let now = marks(chunks);
        let fog = eye.fog_end();
        for mark in self.seen.iter().flat_map(|seen| now.difference(seen)) {
            let (kind, _, id) = *mark;
            let known = self.seen.as_ref().is_some_and(|s| s.contains(&(1, 0, id)));
            if eye.distance(id) < fog && (kind < 2 || known) {
                sample.pops[kind] += 1;
                sample.faded += u32::from(fading(chunks, kind, id));
            }
        }
        self.seen = Some(now);
    }

    /// Notes which requests wait and returns how many and for how long the oldest has.
    fn waits(&mut self, gaps: &[Gap]) -> (usize, u32) {
        let open: HashSet<ChunkId> = gaps.iter().map(|gap| gap.id).collect();
        self.since.retain(|id, _| open.contains(id));
        for id in &open {
            self.since.entry(*id).or_insert(self.frame);
        }
        let oldest = self.since.values().min().copied().unwrap_or(self.frame);
        (open.len(), self.frame - oldest)
    }

    /// Measures one frame of `chunks` seen from `eye`.
    pub fn look(&mut self, chunks: &Chunks, eye: &Eye) -> Sample {
        self.frame += 1;
        let gaps = gaps(chunks, eye);
        let mut sample = Sample {
            ground: nearest(eye, &gaps, |gap| !gap.fine && eye.ahead(gap.id)),
            around: nearest(eye, &gaps, |gap| !gap.fine),
            fine: nearest(eye, &gaps, |gap| gap.fine && eye.ahead(gap.id)),
            ..Sample::default()
        };
        (sample.queue, sample.age) = self.waits(&gaps);
        self.pops(chunks, eye, &mut sample);
        sample
    }
}
