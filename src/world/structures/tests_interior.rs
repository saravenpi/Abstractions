//! Tests of village house interiors: one room light, furniture inside the walls and a free doorway.

use bevy::prelude::*;

use super::Terrain;
use super::brush::{Build, Part, Shape};
use super::glow::LightKind;
use super::site::Kind;
use super::tests::built;
use super::village::House;
use crate::world::Biome;

/// The house kinds a player walks into and lives in.
const HOMES: [House; 8] = [
    House::Cottage,
    House::Long,
    House::TwoStory,
    House::Porch,
    House::Smithy,
    House::Barn,
    House::Tower,
    House::Hall,
];

/// Every biome that has its own village style.
const BIOMES: [Biome; 5] = [
    Biome::Meadow,
    Biome::Taiga,
    Biome::Desert,
    Biome::Tropical,
    Biome::Cherry,
];

/// Yaw `House::draw_variant` gives its lot.
const YAW: f32 = 0.3;

/// Floor height of the site `built` makes.
const FLOOR: f32 = 5.0;

/// Converts a chunk-space point of a house drawn by `draw_house` back to the house frame.
fn local(p: Vec3) -> Vec3 {
    Quat::from_rotation_y(-YAW) * (p - Vec3::Y * FLOOR)
}

/// Returns the low and high corners, in the house frame, of the box around every solid part.
fn solids(b: &Build) -> Vec<(Vec3, Vec3)> {
    let back = Quat::from_rotation_y(-YAW);
    let span = |turn: Quat, shape: Shape| match shape {
        Shape::Cylinder(r, h) | Shape::Cone(r, h) => (Vec3::ZERO, Vec3::new(r, h * 0.5, r)),
        Shape::Cuboid(s) => {
            let m = Mat3::from_quat(turn);
            let half = m.x_axis.abs() * s.x + m.y_axis.abs() * s.y + m.z_axis.abs() * s.z;
            (Vec3::ZERO, half * 0.5)
        }
        Shape::Hull(points) => {
            let turned = points.map(|c| turn * c);
            let lo = turned.into_iter().fold(Vec3::MAX, Vec3::min);
            let hi = turned.into_iter().fold(Vec3::MIN, Vec3::max);
            ((lo + hi) * 0.5, (hi - lo) * 0.5)
        }
    };
    let around = |p: &Part| {
        let (middle, half) = span(back * p.rot, p.shape);
        (local(p.pos) + middle - half, local(p.pos) + middle + half)
    };
    b.parts.iter().map(around).collect()
}

/// Draws `house` in the style of `biome` with the lot variation `variant`.
fn drawn<'a>(t: &'a Terrain, biome: Biome, house: House, variant: u32) -> Build<'a> {
    let mut b = built(t, Kind::Village, variant);
    house.draw_variant(&mut b, biome, variant);
    b
}

/// Runs `check` on every home in every biome style, for several lot variations.
fn every_home(check: impl Fn(&Build, House, &str)) {
    let t = Terrain::new(1);
    for biome in BIOMES {
        for house in HOMES {
            for variant in 0..12 {
                let b = drawn(&t, biome, house, variant * 7919 + 3);
                check(&b, house, &format!("{house:?} in {biome:?} #{variant}"));
            }
        }
    }
}

/// Every home has exactly one candle or lantern and at most one hearth, all inside the room, away from the walls.
#[test]
fn homes_have_one_room_light() {
    every_home(|b, house, what| {
        let s = house.shell();
        let count = |k: LightKind| b.lights.iter().filter(|l| l.kind == k).count();
        assert_eq!(count(LightKind::Candle), 1, "{what}: candles");
        assert!(count(LightKind::Hearth) <= 1, "{what}: hearths");
        let inside = [LightKind::Candle, LightKind::Hearth];
        for l in b.lights.iter().filter(|l| inside.contains(&l.kind)) {
            let p = local(l.pos);
            let side = (s.inner() - Vec2::new(p.x.abs(), p.z.abs())).min_element();
            let clear = side.min(p.y).min(s.h - p.y);
            assert!(
                clear >= 0.299,
                "{what}: light {p} is {clear} m from the room shell"
            );
        }
    });
}

/// Timber cottages get a hearth with their chimney in most lots, and homes without a chimney get none.
#[test]
fn hearths_follow_chimneys() {
    let t = Terrain::new(1);
    let hearths = |biome: Biome, house: House| {
        (0..40)
            .map(|v| drawn(&t, biome, house, v * 7919 + 3))
            .filter(|b| b.lights.iter().any(|l| l.kind == LightKind::Hearth))
            .count()
    };
    assert!((15..40).contains(&hearths(Biome::Meadow, House::Cottage)));
    assert_eq!(hearths(Biome::Desert, House::Cottage), 0);
    assert_eq!(hearths(Biome::Meadow, House::Porch), 0);
    assert_eq!(hearths(Biome::Meadow, House::Smithy), 40);
}

/// Every solid part centered in the room, which is the furniture, lies wholly inside the walls and under the ceiling.
#[test]
fn furniture_stays_in_the_room() {
    every_home(|b, house, what| {
        let s = house.shell();
        let room = Vec3::new(s.inner().x, s.h, s.inner().y);
        let centered = |lo: Vec3, hi: Vec3| {
            let c = (lo + hi) * 0.5;
            c.x.abs() < room.x && c.z.abs() < room.z && c.y > 0.0 && c.y < room.y
        };
        let furniture: Vec<_> = solids(b)
            .into_iter()
            .filter(|(lo, hi)| centered(*lo, *hi))
            .collect();
        assert!(!furniture.is_empty(), "{what}: no solid furniture");
        for (lo, hi) in furniture {
            let fits = lo.x.min(-hi.x) > -room.x - 1e-3 && lo.z.min(-hi.z) > -room.z - 1e-3;
            assert!(
                fits && lo.y > -0.11 && hi.y < room.y + 1e-3,
                "{what}: {lo} to {hi}"
            );
        }
    });
}

/// A corridor 0.7 m wide and 2 m tall, from the doorway 1 m into the room, holds no solid part.
#[test]
fn doorway_is_free() {
    every_home(|b, house, what| {
        let s = house.shell();
        let lo = Vec3::new(s.door - 0.35, 0.01, s.inner().y - 1.0);
        let hi = Vec3::new(s.door + 0.35, 2.0, s.hz);
        for (plo, phi) in solids(b) {
            let apart = plo.cmpge(hi).any() || phi.cmple(lo).any();
            assert!(apart, "{what}: {plo} to {phi} blocks the doorway");
        }
    });
}
