//! Door dressing by style: wall torch, awning with a palm, or lanterns.

use bevy::prelude::*;

use super::shell::{DOOR_H, Shell};
use super::style::{Flavor, Style};
use crate::world::solids::fan;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::palette::{DARK_WOOD, GLASS};

/// Draws an awning over the door and a potted palm beside it.
fn awning(b: &mut Build, s: &Shell, st: &Style) {
    let (d0, d1) = (s.door - s.gap * 0.5 - 0.3, s.door + s.gap * 0.5 + 0.3);
    let rgb = st.cloth[b.site.variant as usize % 3];
    let (lo, hi) = (
        Vec3::new(d0, DOOR_H + 0.4, s.hz),
        Vec3::new(d1, DOOR_H + 0.4, s.hz + 1.3),
    );
    b.lean_to(lo, hi, 0.35, Mat::deco(rgb));
    let at = Vec3::new(d1 + 0.9, 0.0, s.hz + 0.9);
    b.pillar(at, Vec2::new(0.25, 0.45), 6, Mat::deco(DARK_WOOD));
    b.pillar(
        at + Vec3::Y * 0.4,
        Vec2::new(0.07, 1.1),
        4,
        Mat::deco(DARK_WOOD),
    );
    let color = b.color([0.2, 0.5, 0.2], 1.0);
    let top = b.pt(at + Vec3::Y * 1.5);
    fan(&mut b.mesh, top, [6.0, 1.2, 0.5, 0.35, 0.5], 0.3, color);
}

/// Draws two lanterns on posts beside the door.
fn lanterns(b: &mut Build, s: &Shell) {
    let (d0, d1) = (s.door - s.gap * 0.5 - 0.5, s.door + s.gap * 0.5 + 0.5);
    for x in [d0, d1] {
        b.pillar(
            Vec3::new(x, 0.0, s.hz + 0.6),
            Vec2::new(0.06, 2.0),
            4,
            Mat::deco(DARK_WOOD),
        );
        b.cuboid(
            Vec3::new(x - 0.15, 1.9, s.hz + 0.45),
            Vec3::new(x + 0.15, 2.25, s.hz + 0.75),
            Mat::deco(GLASS),
        );
    }
}

/// Draws a wall torch beside the door, plus an awning and a palm, or lanterns, depending on the style.
pub fn trim(b: &mut Build, s: &Shell, st: &Style) {
    b.wall_torch(Vec3::new(s.door + s.gap * 0.5 + 0.5, 1.6, s.hz));
    match st.flavor {
        Flavor::Adobe => awning(b, s, st),
        Flavor::Cherry => lanterns(b, s),
        _ => {}
    }
}
