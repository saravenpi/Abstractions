//! Campsites: layout choice by biome and roll, tents, and the hiker and abandoned layouts.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::dress::{barrel, crate_at, fire, on_ground, seat};
use super::palette::{CANVAS, DARK_WOOD, WOOD};
use super::{camp2, fishing};
use crate::world::{Biome, Surface};

/// The camp layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// One to three tents, a fire with log seats and a cooking tripod.
    Hiker,
    /// Lean-to shelter, drying rack and a boat pulled onto the beach.
    Fishing,
    /// Hide tent, log pile and a stump with an axe.
    Hunter,
    /// Awning on poles with crates and rugs.
    Caravan,
    /// Collapsed tent around a cold fire.
    Abandoned,
}

impl Layout {
    /// Every layout.
    #[cfg(test)]
    pub const ALL: [Layout; 5] = [
        Layout::Hiker,
        Layout::Fishing,
        Layout::Hunter,
        Layout::Caravan,
        Layout::Abandoned,
    ];
}

const TENTS: [[f32; 3]; 4] = [
    CANVAS,
    [0.74, 0.3, 0.22],
    [0.25, 0.5, 0.6],
    [0.8, 0.66, 0.2],
];

/// Returns the direction angle of the nearest open water within 30 m, if any.
fn water_angle(b: &Build) -> Option<f32> {
    let p = b.site.origin;
    let sea = b.terrain.globe().sea_level();
    (0..16).map(|i| i as f32 / 16.0 * TAU).find(|a| {
        [12.0, 20.0, 28.0]
            .iter()
            .any(|m| b.terrain.height(p.x + a.sin() * m, p.y + a.cos() * m) <= sea)
    })
}

/// Chooses the layout for the current site from its biome, ground and a roll.
fn pick(b: &Build) -> Layout {
    let p = b.site.origin;
    let roll = b.roll(7);
    let sandy = b.terrain.surface(p.x, p.y) == Surface::Sand;
    match b.terrain.biome(p.x, p.y) {
        Biome::Desert if roll < 0.75 => Layout::Caravan,
        Biome::Desert => Layout::Abandoned,
        _ if sandy && water_angle(b).is_some() && roll < 0.7 => Layout::Fishing,
        Biome::Forest | Biome::Taiga if roll < 0.45 => Layout::Hunter,
        _ if roll < 0.82 => Layout::Hiker,
        _ => Layout::Abandoned,
    }
}

/// Draws one tent of canvas `rgb` whose open end faces local +x. Its gable is its collider.
pub fn tent(b: &mut Build, off: Vec2, turn: f32, rgb: [f32; 3]) {
    b.frame(off, turn);
    b.settle(1.2, 0.0);
    let rise = 2.3;
    b.gable(
        Vec3::new(-1.2, -0.7, -1.0),
        Vec3::new(1.2, -0.7, 1.0),
        rise,
        Mat::deco(rgb),
    );
    let door = [
        Vec3::new(1.21, 0.0, -0.45),
        Vec3::new(1.21, 0.0, 0.45),
        Vec3::new(1.21, 1.1, 0.0),
    ];
    let color = b.color(DARK_WOOD, 0.6);
    let out = b.dir(Vec3::X);
    b.sheet_tri(door.map(|p| b.pt(p)), out, [color, color]);
}

/// Draws the hiker camp: tents around a fire, seats and a tripod.
fn hiker(b: &mut Build) {
    fire(b, on_ground(b, 0.0, 0.0), false);
    let base = on_ground(b, 0.0, 1.9);
    seat(b, base, 1.6);
    let base = on_ground(b, 0.0, -1.9);
    seat(b, base, 1.6);
    camp2::tripod(b, Vec2::new(1.6, 0.0));
    let count = 1 + (b.roll(1) * 3.0) as usize;
    for i in 0..count {
        let a = 0.6 + i as f32 * 2.1;
        let (rgb, at) = (
            TENTS[(b.site.variant as usize + i) % 4],
            Vec2::new(a.sin(), a.cos()) * 4.2,
        );
        tent(b, at, a.sin().atan2(a.cos()) + PI, rgb);
    }
}

/// Draws the abandoned camp: a collapsed tent, a cold fire and a fallen seat.
fn abandoned(b: &mut Build) {
    fire(b, on_ground(b, 0.0, 0.0), true);
    b.frame(Vec2::new(-3.6, 0.4), 0.3);
    b.settle(1.4, 0.0);
    b.gable(
        Vec3::new(-1.5, 0.0, -1.2),
        Vec3::new(1.5, 0.0, 1.2),
        0.6,
        Mat::deco([0.55, 0.5, 0.4]),
    );
    b.cuboid(
        Vec3::new(-2.0, 0.0, 1.2),
        Vec3::new(1.2, 0.12, 1.34),
        Mat::solid(WOOD),
    );
    b.frame(Vec2::ZERO, 0.0);
    let base = on_ground(b, 1.4, 1.9);
    seat(b, base, 1.4);
    let base = on_ground(b, 2.3, -1.6);
    crate_at(b, base, 0.7);
    barrel(b, on_ground(b, 3.0, -0.6));
}

/// Draws layout `layout` in the current frame.
pub fn draw_layout(b: &mut Build, layout: Layout) {
    match layout {
        Layout::Hiker => hiker(b),
        Layout::Fishing => fishing::fishing(b),
        Layout::Hunter => camp2::hunter(b),
        Layout::Caravan => camp2::caravan(b),
        Layout::Abandoned => abandoned(b),
    }
}

/// Draws the camp of the current site, turned toward the sea when it is a fishing camp.
pub fn camp(b: &mut Build) {
    let layout = match (pick(b), water_angle(b)) {
        (Layout::Fishing, None) => Layout::Hiker,
        (layout, _) => layout,
    };
    if layout == Layout::Fishing {
        let a = water_angle(b).unwrap_or(0.0);
        b.site.yaw = a;
        b.yaw = a;
    }
    draw_layout(b, layout);
}
