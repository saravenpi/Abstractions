//! The deterministic layout of one village: plaza, streets, lots, street ends and fields.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::fields::{Field, fields};
use super::lots::{Lot, Plan, place};
use super::paths::paths_for;
use super::style::{Style, style_for};
use super::trace::street;
use crate::world::Terrain;
use crate::world::structures::site::Site;

/// Radius of the plaza around the center piece.
pub const PLAZA: f32 = 6.5;

/// The piece standing in the middle of the plaza.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Centre {
    /// Stone well with a small roof.
    Well,
    /// Round fountain.
    Fountain,
    /// One big tree ringed by benches.
    TreeRing,
    /// Stone sundial on a stepped base.
    Sundial,
}

/// What stands at the end of a street.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    /// Lamp post.
    Lamp,
    /// Bench.
    Bench,
    /// Hand cart.
    Cart,
    /// Small wayside shrine.
    Shrine,
}

/// A prop closing a street.
#[derive(Debug, Clone, Copy)]
pub struct End {
    /// World position.
    pub pos: Vec2,
    /// Yaw facing back along the street.
    pub yaw: f32,
    /// Which prop.
    pub prop: Prop,
}

/// Everything a village is made of.
pub struct Layout {
    /// The village site.
    pub site: Site,
    /// Palette and tradition.
    pub style: Style,
    /// Plaza center piece.
    pub centre: Centre,
    /// Street polylines, sampled every 2 m, from the plaza outward.
    pub streets: Vec<Vec<Vec2>>,
    /// Building plots.
    pub lots: Vec<Lot>,
    /// Props closing the streets.
    pub ends: Vec<End>,
    /// Crop fields.
    pub fields: Vec<Field>,
    /// Footpaths as world segments: one per lot from its door to a street, in lot order, then the field paths.
    pub paths: Vec<(Vec2, Vec2)>,
}

/// Traces the streets leaving the plaza, sized by the number of buildings wanted.
fn streets_for(
    terrain: &Terrain,
    site: &Site,
    rng: &mut ChaCha8Rng,
    want: usize,
) -> Vec<Vec<Vec2>> {
    let count = (2 + (want - 6) / 3).min(4);
    (0..count)
        .map(|i| {
            let a = site.yaw + i as f32 / count as f32 * TAU + rng.random_range(-0.35..0.35);
            let start = site.origin + Vec2::from_angle(a) * PLAZA;
            let (bend, len) = (rng.random_range(-0.03..0.03), rng.random_range(24.0..44.0));
            street(terrain, start, a, bend, len)
        })
        .filter(|s| s.len() >= 6)
        .collect()
}

/// Picks the piece in the middle of the plaza.
fn pick_centre(rng: &mut ChaCha8Rng) -> Centre {
    [
        Centre::Well,
        Centre::Fountain,
        Centre::TreeRing,
        Centre::Sundial,
    ][rng.random_range(0..4)]
}

/// Builds the layout of the village at `site`. Depends only on the terrain and the site.
pub fn build(terrain: &Terrain, site: Site) -> Layout {
    let mut rng = ChaCha8Rng::seed_from_u64(u64::from(site.variant) ^ 0x0071_11A6_E5ED);
    let style = style_for(terrain.biome(site.origin.x, site.origin.y), site.variant);
    let want = rng.random_range(6..=14usize);
    let streets = streets_for(terrain, &site, &mut rng, want);
    let plan = Plan {
        style,
        center: site.origin,
        want,
    };
    let lots = match streets.is_empty() {
        true => Vec::new(),
        false => place(terrain, &mut rng, &streets, &plan),
    };
    let ends = streets
        .iter()
        .map(|s| end_of(s, rng.random_range(0..4u32)))
        .collect();
    let fields = fields(terrain, &mut rng, &streets, &lots, &plan);
    let centre = pick_centre(&mut rng);
    let paths = paths_for(&streets, &lots, &fields);
    Layout {
        site,
        style,
        centre,
        streets,
        lots,
        ends,
        fields,
        paths,
    }
}

/// Returns the prop closing a street, one pace beyond its last sample.
fn end_of(pts: &[Vec2], roll: u32) -> End {
    let last = pts[pts.len() - 1];
    let dir = (last - pts[pts.len() - 2]).normalize();
    let prop = [Prop::Lamp, Prop::Bench, Prop::Cart, Prop::Shrine][roll as usize];
    End {
        pos: last + dir * 1.6,
        yaw: super::trace::yaw_of(-dir),
        prop,
    }
}
