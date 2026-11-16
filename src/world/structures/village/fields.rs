//! Crop fields in a ring beyond the houses: placement on flat grass, rows, borders, scarecrow, hay and cart.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use super::lots::{Lot, Plan};
use super::props::cart;
use super::style::Flavor;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{fence, hay, on_ground};
use crate::world::structures::palette::{DARK_WOOD, PLANK, ROOF};
use crate::world::{Surface, Terrain};

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

/// Draws the crop rows across the field.
fn rows(b: &mut Build, f: &Field) {
    let n = (f.half.y * 2.0 / 1.1) as i32;
    for i in 0..n {
        let z = -f.half.y + 0.55 + i as f32 * 1.1;
        let base = on_ground(b, 0.0, z).y;
        let (lo, hi) = (
            Vec3::new(-f.half.x, base - 0.2, z - 0.3),
            Vec3::new(f.half.x, base + 0.7, z + 0.3),
        );
        match f.crop {
            Crop::Wheat => {
                b.cuboid(lo, hi, Mat::deco([0.84, 0.68, 0.22]));
                let (tl, th) = (
                    lo + Vec3::new(0.0, 0.9, 0.15),
                    hi + Vec3::new(0.0, 0.25, -0.15),
                );
                b.cuboid(tl, th, Mat::deco([0.93, 0.8, 0.32]));
            }
            Crop::Green => b.cuboid(
                lo,
                hi - Vec3::Y * 0.3,
                Mat::deco([0.3 + 0.1 * (i % 2) as f32, 0.55, 0.2]),
            ),
        }
    }
}

/// Draws a hedge or a fence along both long sides.
fn border(b: &mut Build, f: &Field) {
    let hedge = b.roll(3) < 0.5;
    for sz in [-1.0, 1.0] {
        let z = sz * (f.half.y + 0.5);
        if hedge {
            let base = on_ground(b, 0.0, z).y;
            let (lo, hi) = (
                Vec3::new(-f.half.x, base - 0.2, z - 0.3),
                Vec3::new(f.half.x, base + 0.8, z + 0.3),
            );
            b.cuboid(lo, hi, Mat::deco([0.22, 0.42, 0.18]));
        } else {
            fence(b, -f.half.x, f.half.x, z);
        }
    }
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
    rows(b, f);
    border(b, f);
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
