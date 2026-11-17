//! Crop fields in a ring beyond the houses: placement on flat grass, rows and borders that follow the ground,
//! scarecrow, hay and cart.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use super::lots::{Lot, Plan};
use super::props::cart;
use super::style::Flavor;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{hay, on_ground};
use crate::world::structures::palette::{DARK_WOOD, PLANK, ROOF, WOOD};
use crate::world::{Surface, Terrain};

/// Longest span of a row, hedge or rail drawn at one slope.
const SPAN: f32 = 2.4;

/// How far rows, hedges and posts reach below the ground.
const DIG: f32 = 0.25;

/// Gold of the wheat stalks.
const STALK: [f32; 3] = [0.84, 0.68, 0.22];

/// Pale gold of the wheat ears.
const EAR: [f32; 3] = [0.93, 0.8, 0.32];

/// Dark green of the hedges.
const HEDGE: [f32; 3] = [0.22, 0.42, 0.18];

/// What grows in a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crop {
    /// Golden wheat.
    Wheat,
    /// Green vegetable rows.
    Green,
}

/// A rectangle of crop rows.
#[derive(Debug, Clone, Copy)]
pub struct Field {
    /// World position of the center.
    pub pos: Vec2,
    /// Yaw of the rows.
    pub yaw: f32,
    /// Half length along the rows and half width across them.
    pub half: Vec2,
    /// What grows here.
    pub crop: Crop,
}

/// Returns true when the rectangle of `field` is flat grass.
fn flat(terrain: &Terrain, field: &Field) -> bool {
    let rot = Quat::from_rotation_y(field.yaw);
    let hs: Vec<f32> = [
        (0.0, 0.0),
        (-1.0, -1.0),
        (1.0, -1.0),
        (-1.0, 1.0),
        (1.0, 1.0),
    ]
    .iter()
    .map(|(sx, sz)| {
        let o = rot * Vec3::new(sx * field.half.x, 0.0, sz * field.half.y);
        terrain.height(field.pos.x + o.x, field.pos.y + o.z)
    })
    .collect();
    let (lo, hi) = hs
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    lo > 1.0 && hi - lo < 0.6 && terrain.surface(field.pos.x, field.pos.y) == Surface::Grass
}

/// Returns a random field around the village, not yet checked against the ground.
fn candidate(rng: &mut ChaCha8Rng, center: Vec2) -> Field {
    let a = rng.random_range(0.0..TAU);
    Field {
        pos: center + Vec2::from_angle(a) * rng.random_range(36.0..56.0),
        yaw: rng.random_range(0.0..TAU),
        half: Vec2::new(rng.random_range(3.5..5.5), rng.random_range(2.5..4.0)),
        crop: if rng.random_bool(0.65) {
            Crop::Wheat
        } else {
            Crop::Green
        },
    }
}

/// Places up to six fields in a ring beyond the houses, clear of lots, streets and each other.
pub fn fields(
    terrain: &Terrain,
    rng: &mut ChaCha8Rng,
    streets: &[Vec<Vec2>],
    lots: &[Lot],
    plan: &Plan,
) -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    if matches!(plan.style.flavor, Flavor::Adobe | Flavor::Taiga) {
        return out;
    }
    for _ in 0..30 {
        let f = candidate(rng, plan.center);
        let r = f.half.length();
        let free = lots.iter().all(|l| l.pos.distance(f.pos) > r + 7.0)
            && streets
                .iter()
                .all(|s| s.iter().all(|p| p.distance(f.pos) > r * 0.8 + 4.0))
            && out
                .iter()
                .all(|o| o.pos.distance(f.pos) > r + o.half.length() + 2.0);
        if free && flat(terrain, &f) {
            out.push(f);
        }
        if out.len() == 6 {
            break;
        }
    }
    out
}

/// Draws a bar along local x from `-hx` to `hx` at depth `z` that follows the ground: stations at most `SPAN`
/// apart, each on the lowest visible ground under it. `sect` is the half width, then bottom and top above ground.
fn ribbon(b: &mut Build, hx: f32, z: f32, sect: Vec3, rgb: [f32; 3]) {
    let n = (hx * 2.0 / SPAN).ceil().max(1.0) as u32;
    let station = |b: &Build, i: u32| {
        let x = -hx + hx * 2.0 * i as f32 / n as f32;
        let low = [-sect.x, sect.x].map(|dz| {
            let p = b.pt(Vec3::new(x, 0.0, z + dz));
            b.mesh_y(p.xz() + b.center) - b.base_y
        });
        let at = |dz: f32, y: f32| b.pt(Vec3::new(x, low[0].min(low[1]) + y, z + dz));
        [(-1.0, sect.y), (-1.0, sect.z), (1.0, sect.z), (1.0, sect.y)]
            .map(|(k, y)| at(k * sect.x, y))
    };
    let (top, side) = (b.color(rgb, 1.08), b.color(rgb, 0.88));
    let (front, back) = (b.dir(Vec3::Z), b.dir(Vec3::NEG_Z));
    let mut last = station(b, 0);
    b.quad_out(last, b.dir(Vec3::NEG_X), side);
    for i in 1..=n {
        let next = station(b, i);
        b.quad_out([last[1], last[2], next[2], next[1]], Vec3::Y, top);
        b.quad_out([last[0], last[1], next[1], next[0]], back, side);
        b.quad_out([last[3], last[2], next[2], next[3]], front, side);
        last = next;
    }
    b.quad_out(last, b.dir(Vec3::X), side);
}

/// Draws a fence along local x from `-hx` to `hx` at depth `z`: posts on the visible ground and a rail that follows it.
fn fence(b: &mut Build, hx: f32, z: f32) {
    let n = (hx * 2.0 / SPAN).ceil().max(1.0) as u32;
    for i in 0..=n {
        let x = -hx + hx * 2.0 * i as f32 / n as f32;
        let p = b.pt(Vec3::new(x, 0.0, z));
        let y = b.mesh_y(p.xz() + b.center) - b.base_y;
        let (foot, size) = (Vec3::new(x, y - DIG, z), Vec2::new(0.07, 1.1 + DIG));
        b.pillar(foot, size, 4, Mat::deco(DARK_WOOD));
    }
    ribbon(b, hx, z, Vec3::new(0.04, 0.75, 0.85), WOOD);
}

/// Draws the crop rows and a hedge or a fence along both long sides, every piece following the ground.
pub fn crops(b: &mut Build, f: &Field) {
    b.hug();
    let n = (f.half.y * 2.0 / 1.1) as i32;
    for i in 0..n {
        let z = -f.half.y + 0.55 + i as f32 * 1.1;
        let green = [0.3 + 0.1 * (i % 2) as f32, 0.55, 0.2];
        match f.crop {
            Crop::Wheat => {
                ribbon(b, f.half.x, z, Vec3::new(0.3, -DIG, 0.7), STALK);
                ribbon(b, f.half.x, z, Vec3::new(0.15, 0.7, 0.95), EAR);
            }
            Crop::Green => ribbon(b, f.half.x, z, Vec3::new(0.3, -DIG, 0.4), green),
        }
    }
    let hedge = b.roll(3) < 0.5;
    for z in [-f.half.y - 0.5, f.half.y + 0.5] {
        match hedge {
            true => ribbon(b, f.half.x, z, Vec3::new(0.3, -DIG, 0.8), HEDGE),
            false => fence(b, f.half.x, z),
        }
    }
    b.stiffen();
}

/// Draws a scarecrow on a post at the end of the rows.
fn scarecrow(b: &mut Build, f: &Field) {
    let post = on_ground(b, f.half.x + 0.8, 0.0);
    b.pillar(
        post - Vec3::Y * 0.1,
        Vec2::new(0.05, 1.7),
        4,
        Mat::deco(DARK_WOOD),
    );
    let arms = (Vec3::new(-0.05, 1.2, -0.7), Vec3::new(0.05, 1.28, 0.7));
    b.cuboid(post + arms.0, post + arms.1, Mat::deco(DARK_WOOD));
    let head = (Vec3::new(-0.15, 1.65, -0.15), Vec3::new(0.15, 1.95, 0.15));
    b.cuboid(post + head.0, post + head.1, Mat::deco(PLANK));
    let hat = (Vec3::new(-0.25, 1.95, -0.25), Vec3::new(0.25, 2.05, 0.25));
    b.cuboid(post + hat.0, post + hat.1, Mat::deco(ROOF));
}

/// Draws field number `index`: rows, border, and a scarecrow, hay bales or a cart by turn.
pub fn draw(b: &mut Build, f: &Field, index: usize) {
    b.stand(f.pos, f.yaw);
    crops(b, f);
    match index % 3 {
        0 => scarecrow(b, f),
        1 => {
            hay(b, on_ground(b, f.half.x + 1.0, -1.0));
            hay(b, on_ground(b, f.half.x + 1.2, 0.4));
        }
        _ => {
            b.frame(Vec2::new(f.half.x + 2.0, 0.0), 0.5);
            b.settle(1.0, 0.0);
            cart(b);
        }
    }
}
