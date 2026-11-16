//! Props that close a street: lamp post, bench, hand cart and wayside shrine.

use bevy::prelude::*;

use super::layout::{End, Prop};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{barrel, crate_at, on_ground, seat};
use crate::world::structures::glow::LightKind;
use crate::world::structures::palette::{DARK_WOOD, PLANK, STONE, STONE_DARK, WOOD};

/// Where the lantern hangs from the post arm, in the lamp frame.
const LANTERN: Vec3 = Vec3::new(0.0, 2.92, 0.5);

/// Dark iron of the lantern frame.
const IRON: [f32; 3] = [0.13, 0.12, 0.12];

/// Draws a lantern whose flame is centered on the local point `at`: a dark frame around an open core.
///
/// Four thin corner posts stand on two bottom bars under a plate, a pyramid cap and a ring on top.
/// The bottom stays open so the flame lights the ground under the lamp.
fn lantern(b: &mut Build, at: Vec3) {
    let mut bar = |lo: [f32; 3], hi: [f32; 3]| {
        b.cuboid(at + Vec3::from(lo), at + Vec3::from(hi), Mat::deco(IRON));
    };
    for z in [-0.13, 0.13] {
        bar([-0.15, -0.2, z - 0.02], [0.15, -0.16, z + 0.02]);
        for x in [-0.13, 0.13] {
            bar([x - 0.02, -0.16, z - 0.02], [x + 0.02, 0.22, z + 0.02]);
        }
    }
    bar([-0.18, 0.22, -0.18], [0.18, 0.25, 0.18]);
    bar([-0.035, 0.39, -0.035], [0.035, 0.48, 0.035]);
    b.spire(at + Vec3::Y * 0.25, Vec2::new(0.25, 0.16), 4, IRON);
}

/// Draws a lamp post with a short arm and a lantern hanging from it.
fn lamp(b: &mut Build) {
    b.pillar(
        Vec3::Y * -0.1,
        Vec2::new(0.09, 3.6),
        5,
        Mat::deco(DARK_WOOD),
    );
    b.cuboid(
        Vec3::new(-0.04, 3.38, -0.12),
        Vec3::new(0.04, 3.46, LANTERN.z + 0.08),
        Mat::deco(DARK_WOOD),
    );
    lantern(b, LANTERN);
    b.glow(LightKind::Lamp, LANTERN);
}

/// Draws a street lamp standing on the ground at the world point `pos`.
pub fn lamp_at(b: &mut Build, pos: Vec2) {
    b.stand(pos, 0.0);
    lamp(b);
}

/// Draws a bench with a backrest.
fn bench(b: &mut Build) {
    let base = on_ground(b, 0.0, 0.0);
    seat(b, base, 1.8);
    b.cuboid(
        Vec3::new(-0.9, 0.4, -0.3),
        Vec3::new(0.9, 0.9, -0.2),
        Mat::deco(WOOD),
    );
}

/// Draws a hand cart with two wheels, a handle, a barrel and a crate.
pub fn cart(b: &mut Build) {
    b.cuboid(
        Vec3::new(-0.8, 0.35, -0.5),
        Vec3::new(0.8, 0.75, 0.5),
        Mat::solid(PLANK),
    );
    b.pillar(
        Vec3::new(-0.9, 0.0, -0.5),
        Vec2::new(0.35, 0.1),
        8,
        Mat::deco(DARK_WOOD),
    );
    b.pillar(
        Vec3::new(-0.9, 0.0, 0.5),
        Vec2::new(0.35, 0.1),
        8,
        Mat::deco(DARK_WOOD),
    );
    b.cuboid(
        Vec3::new(0.8, 0.45, -0.05),
        Vec3::new(1.9, 0.52, 0.05),
        Mat::deco(DARK_WOOD),
    );
    barrel(b, on_ground(b, 1.4, 1.2));
    crate_at(b, on_ground(b, -1.4, 1.2), 0.6);
}

/// Draws a small stone wayside shrine under a gable.
fn shrine(b: &mut Build) {
    b.cuboid(
        Vec3::new(-0.7, -0.1, -0.7),
        Vec3::new(0.7, 0.5, 0.7),
        Mat::solid(STONE),
    );
    b.cuboid(
        Vec3::new(-0.45, 0.5, -0.45),
        Vec3::new(0.45, 1.5, 0.45),
        Mat::solid(STONE),
    );
    b.gable(
        Vec3::new(-0.8, 1.5, -0.8),
        Vec3::new(0.8, 1.5, 0.8),
        0.6,
        Mat::deco(STONE_DARK),
    );
}

/// Draws the prop that closes a street.
pub fn end_prop(b: &mut Build, end: &End) {
    b.stand(end.pos, end.yaw);
    for x in [-1.5, 1.5] {
        let base = on_ground(b, x, 0.3);
        b.torch_post(base);
    }
    match end.prop {
        Prop::Lamp => lamp(b),
        Prop::Bench => bench(b),
        Prop::Cart => cart(b),
        Prop::Shrine => shrine(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Terrain;
    use crate::world::structures::audit::positions;
    use crate::world::structures::site::Kind;
    use crate::world::structures::tests::built;

    /// Lantern lights sit inside their frame: iron above, below and on four sides, none inside the flame.
    #[test]
    fn lantern_lights_sit_inside_their_frame() {
        let terrain = Terrain::new(1);
        let mut b = built(&terrain, Kind::Village, 3);
        lamp_at(&mut b, Vec2::new(3.0, -2.0));
        assert_eq!(b.lights.len(), 1);
        let (flame, mesh) = (b.lights[0].pos, b.mesh.clone().into_mesh());
        let frame: Vec<Vec3> = positions(&mesh)
            .into_iter()
            .map(|p| p - flame)
            .filter(|p| p.xz().length() < 0.3 && p.y.abs() < 0.3)
            .collect();
        assert!(
            frame
                .iter()
                .all(|p| p.xz().length() > 0.1 || p.y.abs() > 0.14),
            "iron inside the flame"
        );
        assert!(frame.iter().any(|p| p.y > 0.2) && frame.iter().any(|p| p.y < -0.15));
        let corners = |x: f32, z: f32| frame.iter().any(|p| p.x * x > 0.1 && p.z * z > 0.1);
        assert!(
            corners(1.0, 1.0) && corners(1.0, -1.0) && corners(-1.0, 1.0) && corners(-1.0, -1.0)
        );
    }
}
