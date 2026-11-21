//! One pass over a chunk: random tries, each planted with what the biome, surface and density allow there.

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

pub use super::ctx::Ctx;
use super::density::choose;
use super::sow::cost;
use super::spacing::{crowded, ground, rock, room, sunk};
use super::{BIG_CAP, Placement, SMALL_CAP, TRI_BUDGET, deep};
use crate::world::lands::thrive::{frozen, vigor};
use crate::world::propmesh::{Plant, VARIANTS};
use crate::world::{Biome, CHUNK_SIZE, SEA_LEVEL, Surface};

/// Highest ground a willow may stand on.
const WILLOW_CEILING: f32 = 5.0;

/// Distance from a willow to the pond it leans over, in metres.
const WILLOW_REACH: f32 = 6.0;

/// The ground under one try.
pub(super) struct Ground {
    /// Height every decision reads: the visible mesh on a globe, the analytic ground on the legacy plane.
    pub(super) height: f32,
    /// True slope of the visible mesh, rise over run. Zero on the legacy plane, which measures its own.
    pub(super) slope: f32,
    /// What covers the ground.
    pub(super) surface: Surface,
    /// Biome of the spot.
    pub(super) biome: Biome,
}

impl Ctx<'_> {
    /// Returns true when a pond floor lies on a circle of `WILLOW_REACH` metres around `w`.
    fn near_pond(&self, w: Vec2) -> bool {
        (0..6).any(|k| {
            let turn = Vec2::from_angle(k as f32 / 6.0 * std::f32::consts::TAU);
            let q = w + self.metric.chart(turn * WILLOW_REACH);
            self.terrain.pond_at(q.x, q.y) && self.terrain.height(q.x, q.y) <= SEA_LEVEL
        })
    }

    /// Returns the ground at chart position `w`, or nothing where the floor is too deep for anything. On a
    /// globe a channel bed above the sea covers the ground as the terrain mesh paints it, so nothing grows
    /// in a river.
    pub(super) fn look(&self, w: Vec2) -> Option<Ground> {
        let (x, z) = (w.x, w.y);
        let (deep, lands) = (deep(self.terrain), self.terrain.globe().lands());
        let Some(relief) = &self.relief else {
            let height = self.terrain.height(x, z);
            return (height >= deep).then(|| Ground {
                height,
                slope: 0.0,
                surface: self.terrain.surface(x, z),
                biome: self.terrain.biome(x, z),
            });
        };
        let (height, grade) = relief.at(w);
        (height >= deep).then(|| {
            let (slope, biome) = (self.metric.slope(grade), self.terrain.biome(x, z));
            let pond = height <= SEA_LEVEL && self.terrain.pond_at(x, z);
            let bed = (height > SEA_LEVEL)
                .then(|| self.terrain.bed(x, z))
                .flatten();
            let surface = match pond {
                true => Surface::Pond,
                false => bed.unwrap_or_else(|| lands.surface(height, slope, biome)),
            };
            Ground {
                height,
                slope,
                surface,
                biome,
            }
        })
    }

    /// Returns the plant the roll lands on at `w` and the ground it grows on, if any. A sea floor only
    /// carries something where the sea has life. Dry land grows as thick as its climate lets it, and
    /// only stones lie above the snow line.
    fn pick(&self, w: Vec2, roll: f32, big: bool) -> Option<(Plant, Ground)> {
        let at = self.look(w)?;
        let table = self.terrain.globe().lands().plants(at.biome, at.surface);
        let mut density = self.fields.density((self.terrain, w), -at.height);
        let dry = at.height > self.terrain.globe().sea_level();
        if let Some(air) = self.air.as_ref().filter(|_| dry).map(|air| air.at(w)) {
            (density.vigor, density.frozen) = (vigor(at.biome, air), frozen(at.biome, air));
        }
        choose(table, roll, &density, big)
            .filter(|&plant| {
                plant != Plant::Willow || (at.height < WILLOW_CEILING && self.near_pond(w))
            })
            .map(|plant| (plant, at))
    }

    /// Returns the height a prop of scale `scale` stands at: the visible terrain mesh, with rocks sunk into slopes.
    pub(super) fn rest(&self, w: Vec2, p: (Plant, f32), at: &Ground) -> f32 {
        if self.relief.is_none() {
            return ground(self.terrain, w, p, at.height);
        }
        let dry_rock = at.height >= self.terrain.globe().sea_level() && rock(p.0);
        at.height - f32::from(dry_rock) * sunk(at.slope, p.1)
    }

    /// Returns the prop the roll plants at chunk position `local`, if any, with its look drawn from `rng`: from
    /// the flora of the chunk when it has one, else from the tables of home.
    fn place(&self, rng: &mut ChaCha8Rng, local: Vec2, roll: f32, big: bool) -> Option<Placement> {
        let w = self.center + local;
        if self.relief.as_ref().is_some_and(|relief| !relief.holds(w)) {
            return None;
        }
        if let Some(flora) = &self.flora {
            return flora.place(self, rng, (w, local), (roll, big));
        }
        let (plant, at) = self.pick(w, roll, big)?;
        let (plant, lean) = self.settle(rng, w, plant)?;
        let yaw = rng.random_range(0.0..std::f32::consts::TAU);
        let scale = size(rng, plant, at.surface, at.height);
        if at.height < self.terrain.globe().sea_level() && -at.height < plant.draft() * scale {
            return None;
        }
        let variant = rng.random_range(0..VARIANTS) as u8;
        Some(Placement {
            pos: Vec3::new(local.x, self.rest(w, (plant, scale), &at), local.y),
            yaw,
            scale,
            plant,
            variant,
            lean,
        })
    }
}

/// Returns the scale of a new plant, or the height of a kelp, which stops one to two meters under the surface.
pub(super) fn size(rng: &mut ChaCha8Rng, plant: Plant, surface: Surface, height: f32) -> f32 {
    if plant == Plant::Kelp {
        return (-height - 1.0 - rng.random_range(0.0..1.5)).max(1.5);
    }
    let boulder = 1.0 + f32::from(surface == Surface::Rock) * height.max(0.0) * 0.06;
    rng.random_range(0.8..1.3) * boulder
}

/// Returns true when big prop `p` stands clear of the ones already placed, and then remembers where it stands
/// and the room it keeps, in true metres.
fn clear(ctx: &Ctx, spots: &mut Vec<(Vec3, f32)>, p: &Placement) -> bool {
    let local = p.pos.xz();
    let spot = ctx.metric.spot(ctx.terrain, ctx.center + local, local);
    let free = !crowded(spots, spot, room(p));
    if free {
        spots.push((spot, room(p)));
    }
    free
}

/// Draws random points of the chunk, `tries` of them per square of `CHUNK_SIZE` metres, and plants what the
/// biome, surface and density allow there, keeping big props apart. Tries and caps follow the true area.
pub fn pass(ctx: &Ctx, rng: &mut ChaCha8Rng, tries: usize, big: bool) -> Vec<Placement> {
    let half = CHUNK_SIZE * 0.5;
    let cap = ctx.metric.scaled([SMALL_CAP, BIG_CAP][big as usize]);
    let budget = ctx.metric.scaled(TRI_BUDGET);
    let (mut out, mut spots, mut spent) = (Vec::new(), Vec::new(), 0);
    for _ in 0..ctx.metric.scaled(tries) {
        let local = Vec2::new(rng.random_range(-half..half), rng.random_range(-half..half));
        let roll: f32 = rng.random();
        if out.len() >= cap || spent >= budget {
            break;
        }
        let Some(placement) = ctx.place(rng, local, roll, big) else {
            continue;
        };
        if big && !clear(ctx, &mut spots, &placement) {
            continue;
        }
        spent += if big { 0 } else { cost(&placement) };
        out.push(placement);
    }
    out
}
