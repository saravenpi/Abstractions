//! Flow traces: from a spring on high ground, step by step down the region grids to the sea or a basin.
//!
//! Every step goes to the lowest of eight neighbours 125 m away, and only downhill, so a trace never loops
//! and its bed always falls. A trace that has not ended after 40 steps, 5 km, is dropped, which
//! keeps every flow close to the cell it starts in.

use std::f64::consts::FRAC_PI_4;

use bevy::math::DVec3;

use super::Cell;
use super::channel::Flowing;
use crate::world::SEA_LEVEL;
use crate::world::globe::Globe;
use crate::world::globe::climate::frame;
use crate::world::globe::roll::roll;
use crate::world::globe::season::MEAN_SEASON;

/// Spring candidates tried in a cell of 2 km by 2 km, fewer in a smaller one.
const TRIES: f64 = 64.0;

/// Metres between two steps of a trace.
pub const STEP: f64 = 125.0;

/// Most steps a trace takes before it is dropped.
pub const LONGEST: usize = 40;

/// Fewest steps of a flow that is kept.
const SHORTEST: usize = 3;

/// Least fall in metres from one step to the next.
const DROP: f32 = 0.05;

/// Lowest ground in metres above sea level a flow springs from.
const HIGH: f32 = 30.0;

/// Least wetness of the climate at a river spring.
const WET: f32 = 0.25;

/// One point of a flow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
    /// Unit direction of the point.
    pub dir: DVec3,
    /// Height of the region grid there.
    pub height: f32,
    /// Height of the channel bed there.
    pub bed: f32,
    /// Half width of the channel there, in metres.
    pub half: f32,
}

/// Where a flow ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// At or under the sea level.
    Sea,
    /// In a hollow every neighbour of which is higher.
    Basin,
}

/// One flow, from its spring down to its end.
#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    /// The points of the flow, spring first.
    pub nodes: Vec<Node>,
    /// Where it ends.
    pub end: End,
}

/// Returns the flows that spring in `cell` of `globe`.
pub fn sources(globe: &Globe, cell: Cell) -> Vec<Flow> {
    let kind = globe.flowing();
    if kind == Flowing::Nothing {
        return Vec::new();
    }
    let key = [cell.face as i64, i64::from(cell.i), i64::from(cell.j)];
    let spot = |k: u32| [0, 1].map(|s| 1.0 + 14.0 * f64::from(roll(globe.seed(), key, 2 * k + s)));
    let [wide, tall] = cell.size(globe.cells(), globe.radius());
    let tries = (TRIES * wide * tall / 4e6).ceil() as u32;
    (0..tries)
        .map(|k| cell.dir(spot(k), globe.cells()))
        .filter(|&dir| springs(globe, dir, kind))
        .filter_map(|dir| trace(globe, dir, kind))
        .collect()
}

/// Returns true when a flow of `kind` springs in unit direction `dir`: high ground, and for a river a wet
/// climate in the annual mean.
fn springs(globe: &Globe, dir: DVec3, kind: Flowing) -> bool {
    let high = globe.shaped(dir * globe.radius()) >= HIGH;
    high && (kind == Flowing::Lava || globe.climate_in(dir, MEAN_SEASON).moisture >= WET)
}

/// Returns the point of a flow of `kind` at `dir`, `step` steps from its spring, the grid being `height`
/// high there.
fn node(kind: Flowing, dir: DVec3, height: f32, step: usize) -> Node {
    let (half, depth) = kind.size(step);
    Node {
        dir,
        height,
        bed: height - depth,
        half,
    }
}

/// Returns the lowest of the eight neighbours one step from `dir`, and its height.
pub fn downhill(globe: &Globe, dir: DVec3) -> (DVec3, f32) {
    let (east, north) = frame(dir);
    let reach = STEP / globe.radius();
    (0..8)
        .map(|k| {
            let (sin, cos) = (f64::from(k) * FRAC_PI_4).sin_cos();
            let to = (dir + (east * cos + north * sin) * reach).normalize();
            (to, globe.coarse(to))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or((dir, f32::INFINITY))
}

/// Returns the flow of `kind` from the spring at `start`, or nothing when it is too short or too long.
fn trace(globe: &Globe, start: DVec3, kind: Flowing) -> Option<Flow> {
    let mut nodes = vec![node(kind, start, globe.coarse(start), 0)];
    for step in 1..=LONGEST {
        let last = nodes[nodes.len() - 1];
        let (dir, height) = downhill(globe, last.dir);
        if height > last.height - DROP {
            return (step > SHORTEST).then_some(Flow {
                nodes,
                end: End::Basin,
            });
        }
        nodes.push(node(kind, dir, height, step));
        if height <= SEA_LEVEL {
            return (step >= SHORTEST).then_some(Flow {
                nodes,
                end: End::Sea,
            });
        }
    }
    None
}
