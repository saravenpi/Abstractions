//! Door dressing by style: wall torch, awning with a palm, or lanterns. Props off the plinth stand on the ground.

use bevy::prelude::*;

use super::shell::Shell;
use super::style::{Flavor, Style};
use crate::world::solids::fan;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::on_ground;
use crate::world::structures::palette::{DARK_WOOD, GLASS};
use crate::world::structures::tags::Tag;

/// Returns where a prop of radius `r` at local `(x, z)` stands: on the plinth when it fits on it, else on the ground.
fn foot(b: &Build, s: &Shell, at: Vec2, r: f32) -> Vec3 {
    let on_plinth = at.x.abs() + r <= s.hx + 0.5 && at.y + r <= s.hz + s.front + 0.5;
    match on_plinth {
        true => Vec3::new(at.x, 0.0, at.y),
        false => on_ground(b, at.x, at.y) - Vec3::Y * 0.1,
    }
}

/// Draws an awning over the door, held by the wall, and a potted palm beside it.
fn awning(b: &mut Build, s: &Shell, st: &Style) {
    let (d0, d1) = (s.door - s.gap * 0.5 - 0.3, s.door + s.gap * 0.5 + 0.3);
    let rgb = st.cloth[b.site.variant as usize % 3];
    let (lo, hi) = (
        Vec3::new(d0, s.door_h + 0.15, s.hz),
        Vec3::new(d1, s.door_h + 0.15, s.hz + 1.3),
    );
    b.lean_to(lo, hi, 0.35, Mat::deco(rgb));
    let at = foot(b, s, Vec2::new(d1 + 0.9, s.hz + 0.9), 0.25);
    b.pillar(at, Vec2::new(0.25, 0.45), 6, Mat::solid(DARK_WOOD));
    b.pillar(
        at + Vec3::Y * 0.4,
        Vec2::new(0.07, 1.1),
        4,
        Mat::deco(DARK_WOOD),
    );
    let color = b.color([0.2, 0.5, 0.2], 1.0);
    let top = b.pt(at + Vec3::Y * 1.5);
    b.tagged(Tag::Soft, |b| {
        fan(&mut b.mesh, top, [6.0, 1.2, 0.5, 0.35, 0.5], 0.3, color)
    });
}

/// Draws two lanterns on posts beside the door; a post off the plinth reaches down to the ground.
fn lanterns(b: &mut Build, s: &Shell) {
    let (d0, d1) = (s.door - s.gap * 0.5 - 0.5, s.door + s.gap * 0.5 + 0.5);
    for x in [d0, d1] {
        let at = foot(b, s, Vec2::new(x, s.hz + 0.6), 0.06);
        b.pillar(at, Vec2::new(0.06, 2.0 - at.y), 4, Mat::deco(DARK_WOOD));
        let glass = (
            Vec3::new(x - 0.15, 1.9, s.hz + 0.45),
            Vec3::new(x + 0.15, 2.25, s.hz + 0.75),
        );
        b.tagged(Tag::Soft, |b| b.cuboid(glass.0, glass.1, Mat::deco(GLASS)));
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
