//! Flow traces: from a spring on high ground, step by step down the region grids to the sea or a basin.
//!
//! Every step goes to the lowest of eight neighbours 125 m away, and only downhill, so a trace never loops
//! and its bed always falls. A trace that has not ended after 40 steps, 5 km, is dropped, which
//! keeps every flow close to the cell it starts in.
//!
//! The trunks of the drainage grid are the exception: a flow that follows one is steered towards the next
//! cell of the trunk and kept at its longest, full width from its first step, since the springs of one
//! trunk lie 2 km apart and their flows overlap into one river that runs hundreds of kilometres.

use std::f64::consts::FRAC_PI_4;

use bevy::math::DVec3;

use super::Cell;
use super::channel::Flowing;
use super::trunk::trunks;
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

/// Steps of growth a trunk flow starts with, so that it begins at its full width.
pub const TRUNK: usize = 20;

/// Least share of a step that must point towards the goal of a steered trace.
const TOWARDS: f64 = 0.34;

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
        .filter_map(|dir| trace(globe, dir, kind, None))
        .chain(trunks(globe, cell, kind))
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

/// Returns the eight neighbours one step from `dir` and the height of each.
fn neighbours(globe: &Globe, dir: DVec3) -> [(DVec3, f32); 8] {
    let (east, north) = frame(dir);
    let reach = STEP / globe.radius();
    std::array::from_fn(|k| {
        let (sin, cos) = (k as f64 * FRAC_PI_4).sin_cos();
        let to = (dir + (east * cos + north * sin) * reach).normalize();
        (to, globe.coarse(to))
    })
}

/// Returns the lowest of `around` and its height.
fn lowest(around: &[(DVec3, f32)]) -> (DVec3, f32) {
    let low = around.iter().copied().min_by(|a, b| a.1.total_cmp(&b.1));
    low.unwrap_or((DVec3::ZERO, f32::INFINITY))
}

/// Returns the lowest of the eight neighbours one step from `dir`, and its height.
pub fn downhill(globe: &Globe, dir: DVec3) -> (DVec3, f32) {
    lowest(&neighbours(globe, dir))
}

/// Returns the next point of a trace from `dir`: the lowest neighbour, or with a `goal` the lowest of those
/// that fall and point towards it, and when none does, the lowest of all.
fn step_from(globe: &Globe, dir: DVec3, goal: Option<DVec3>) -> (DVec3, f32) {
    let Some(goal) = goal else {
        return downhill(globe, dir);
    };
    let around = neighbours(globe, dir);
    let (here, heading) = (
        globe.coarse(dir),
        (goal - dir * dir.dot(goal)).normalize_or_zero(),
    );
    let ahead = |&(to, height): &(DVec3, f32)| {
        height < here - DROP && (to - dir).normalize().dot(heading) > TOWARDS
    };
    let falling: Vec<(DVec3, f32)> = around.iter().copied().filter(ahead).collect();
    match falling.is_empty() {
        true => lowest(&around),
        false => lowest(&falling),
    }
}

/// Returns the flow of `kind` from the spring at `start`, or nothing when it is too short or too long. A
/// trace with a `goal` is a trunk flow: it is kept at its longest, and starts at full width.
pub fn trace(globe: &Globe, start: DVec3, kind: Flowing, goal: Option<DVec3>) -> Option<Flow> {
    let grown = if goal.is_some() { TRUNK } else { 0 };
    let mut nodes = vec![node(kind, start, globe.coarse(start), grown)];
    for step in 1..=LONGEST {
        let last = nodes[nodes.len() - 1];
        let (dir, height) = step_from(globe, last.dir, goal);
        if height > last.height - DROP {
            return (step > SHORTEST).then_some(Flow {
                nodes,
                end: End::Basin,
            });
        }
        nodes.push(node(kind, dir, height, step + grown));
        if height <= SEA_LEVEL {
            return (step >= SHORTEST).then_some(Flow {
                nodes,
                end: End::Sea,
            });
        }
    }
    goal.map(|_| Flow {
        nodes,
        end: End::Basin,
    })
}
