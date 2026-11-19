//! Sowing a planet with a flora: the stones it keeps whatever lives there, then its producer groups by where
//! they live, the climate they stand and the patch density.

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use super::Placement;
use super::density::Density;
use super::grow::{Flora, Grower};
use super::pass::{Ctx, Ground, size};
use crate::climate::Climate;
use crate::life::flora::KELP;
use crate::life::{Habitat, Tolerance};
use crate::world::Surface;
use crate::world::propmesh::grown::{grown, tris};
use crate::world::propmesh::{BIG, Plant, VARIANTS};
use crate::world::props::small_tris;

/// Kelvin outside its tolerance at which a group no longer grows.
const WARMTH: f32 = 8.0;

/// Moisture outside its tolerance at which a group no longer grows.
const WETNESS: f32 = 0.15;

/// Metres from the shore line within which a shore group grows, on either side.
const SHORE: f32 = 2.0;

/// What a roll lands on.
enum Pick<'a> {
    /// A stone of the table of home.
    Stone(Plant),
    /// A producer group.
    Group(&'a Grower),
}

/// Returns true for the stones and boulders a planet keeps whatever lives on it.
fn mineral(p: Plant) -> bool {
    use Plant::*;
    matches!(
        p,
        Rock | IceRock | Sandstone | Pebbles | SmoothRock | Boulder
    )
}

/// Returns how well tolerance `t` stands `air`, from 1 inside it to 0 a few kelvin or a little moisture
/// outside. Without a climate every group stands it.
fn fit(t: &Tolerance, air: Option<&Climate>) -> f32 {
    let Some(air) = air else {
        return 1.0;
    };
    let heat = (t.cold - air.temperature)
        .max(air.temperature - t.heat)
        .max(0.0);
    let damp = (t.dry - air.moisture).max(air.moisture - t.wet).max(0.0);
    (1.0 - heat / WARMTH).max(0.0) * (1.0 - damp / WETNESS).max(0.0)
}

/// Returns how much grower `g` gets on `at`, under the liquid when `under`, at patch density `d`: nothing
/// where it cannot stand, the grove noise for the big, the bloom noise on land and the patch noise below.
fn gain(g: &Grower, at: &Ground, under: bool, d: &Density) -> f32 {
    let water = g.arch == KELP || g.habitat == Habitat::Water;
    let shore = g.habitat == Habitat::Shore && at.height.abs() < SHORE;
    let stands = match (under, at.surface) {
        (true, _) => water || shore,
        (false, Surface::River | Surface::Pond) => false,
        (false, Surface::Rock) => !water && matches!(g.arch, 1 | 2 | 7),
        (false, _) => !water,
    };
    let patch = match (under, g.big()) {
        (true, _) => d.patch.min(1.0),
        (false, true) => d.grove,
        (false, false) => 0.5 + 0.5 * d.bloom,
    };
    f32::from(u8::from(stands)) * patch
}

/// Returns the number of triangles small placement `p` adds to the ground cover, a grown look included.
pub fn cost(p: &Placement) -> usize {
    if grown(p.variant) {
        tris(p.variant)
    } else {
        small_tris(p)
    }
}

/// Returns the scale of a group planted at `depth` metres under the liquid (negative on land), or nothing
/// when it would break the surface. Kelp stops one to two metres under it; sizes spread with the layers.
fn scale(g: &Grower, rng: &mut ChaCha8Rng, depth: f32) -> Option<f32> {
    let reference = g.habit.reference;
    let s = if g.arch == KELP {
        (depth - 1.0 - rng.random_range(0.0..1.5)).max(1.5) / reference
    } else {
        let spread = 0.12 * f32::from(g.layers);
        g.height / reference * rng.random_range(1.0 - spread..1.0 + spread)
    };
    (depth < 0.0 || s * reference < depth - 0.5).then_some(s)
}

/// Returns stone `plant` as it lies on `at`, `depth` metres under the liquid (negative on land), with its
/// scale and variant drawn from `rng`, or nothing where the water is too shallow to hide it.
fn stone(rng: &mut ChaCha8Rng, plant: Plant, at: &Ground, depth: f32) -> Option<(Plant, f32, u8)> {
    let scale = size(rng, plant, at.surface, at.height);
    let variant = rng.random_range(0..VARIANTS) as u8;
    (depth <= 0.0 || depth >= plant.draft() * scale).then_some((plant, scale, variant))
}

impl Flora {
    /// Returns what a roll in `[0, 1)` lands on at chart position `w` over `at` among the big or the small:
    /// a stone of the table of home, then a group, each with its chance.
    fn choose(
        &self,
        ctx: &Ctx,
        (w, at): (Vec2, &Ground),
        roll: f32,
        big: bool,
    ) -> Option<Pick<'_>> {
        let lands = ctx.terrain.globe().lands();
        let mut acc = 0.0;
        let stones = lands.plants(at.biome, at.surface).iter();
        for &(plant, chance) in stones.filter(|(p, _)| mineral(*p) && ((*p as usize) < BIG) == big)
        {
            acc += chance;
            if roll < acc {
                return Some(Pick::Stone(plant));
            }
        }
        let under = at.height < ctx.terrain.globe().sea_level();
        let (d, air) = (
            ctx.fields.density((ctx.terrain, w), -at.height),
            ctx.air.as_ref().map(|a| a.at(w)),
        );
        let growing = self.growers.iter().filter(|g| g.big() == big);
        growing
            .map(|g| {
                (
                    g,
                    g.chance * fit(&g.tolerance, air) * gain(g, at, under, &d),
                )
            })
            .find(|(_, chance)| {
                acc += chance;
                roll < acc
            })
            .map(|(g, _)| Pick::Group(g))
    }

    /// Returns what `roll` plants at chart position `w`, chunk position `local`, among the big or the small,
    /// if anything, with its look drawn from `rng`.
    pub fn place(
        &self,
        ctx: &Ctx,
        rng: &mut ChaCha8Rng,
        (w, local): (Vec2, Vec2),
        (roll, big): (f32, bool),
    ) -> Option<Placement> {
        let at = ctx.look(w)?;
        let pick = self.choose(ctx, (w, &at), roll, big)?;
        let yaw = rng.random_range(0.0..std::f32::consts::TAU);
        let depth = ctx.terrain.globe().sea_level() - at.height;
        let (plant, scale, variant) = match pick {
            Pick::Stone(plant) => stone(rng, plant, &at, depth)?,
            Pick::Group(g) => (
                g.habit.carrier,
                scale(g, rng, depth)?,
                g.looks[rng.random_range(0..VARIANTS)],
            ),
        };
        Some(Placement {
            pos: Vec3::new(local.x, ctx.rest(w, (plant, scale), &at), local.y),
            yaw,
            scale,
            plant,
            variant,
        })
    }
}
