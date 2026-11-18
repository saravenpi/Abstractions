//! Tests that roofs are solid where they are drawn: a ray down onto a roof stops on its drawn surface, and
//! rooms stay free under their roofs.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::audit::positions;
use super::brush::{Build, Mat};
use super::gaps::column;
use super::site::Kind;
use super::tags::Tag;
use super::tests::built;
use super::village::style::style_for;
use super::village::{ALL_HOUSES, House};
use super::{Terrain, compound};
use crate::world::Biome;

/// Color of the roofs drawn alone.
const GREY: [f32; 3] = [0.5, 0.5, 0.5];

/// A roof drawn alone: its name, the low and high corners of its box and its rise.
type Roof = (&'static str, [f32; 3], [f32; 3], f32);

/// Every roof kind, in the sizes the game draws: the shrine hall, a tent, a cottage roof, the fishing
/// shelter, a door awning, the caravan awning, an adobe roof and a hipped cottage roof.
const ROOFS: [Roof; 8] = [
    ("gable", [-2.1, 2.0, -1.8], [2.1, 2.0, 1.6], 1.2),
    ("tent", [-1.2, -0.7, -1.0], [1.2, -0.7, 1.0], 2.3),
    ("slopes", [-3.3, 2.7, -2.7], [3.3, 2.7, 2.7], 1.6),
    ("slope", [-2.6, 1.8, -2.7], [2.6, 1.8, 0.5], 1.0),
    ("lean-to", [-1.0, 2.45, 2.2], [1.0, 2.45, 3.5], 0.35),
    ("awning", [-2.9, 2.3, -2.1], [2.9, 2.3, 2.1], 0.5),
    ("flat", [-3.3, 2.7, -2.7], [3.3, 2.7, 2.7], 0.0),
    ("hip", [-3.3, 2.7, -2.7], [3.3, 2.7, 2.7], 1.6),
];

/// Draws `roof` alone on `b`.
fn draw(b: &mut Build, roof: Roof) {
    let (lo, hi) = (Vec3::from_array(roof.1), Vec3::from_array(roof.2));
    let mat = Mat::deco(GREY);
    match roof.0 {
        "gable" | "tent" => b.gable(lo, hi, roof.3, mat),
        "slopes" => b.slopes(lo, hi, roof.3, mat),
        "slope" => b.slope(lo, hi, roof.3, mat),
        "lean-to" => b.lean_to(lo, hi, roof.3, mat),
        "awning" => b.awning(lo, hi, roof.3, &[GREY, [1.0; 3], GREY]),
        "flat" => b.flat_roof(lo, hi, mat),
        _ => b.hip(lo, hi, roof.3, mat),
    }
}

/// Returns the compound collider of everything solid in `b`.
fn solid(b: &Build) -> Collider {
    compound(&b.parts).expect("the build has colliders")
}

/// Returns the roof triangles of `b` that face up, in chunk space.
fn roofing(b: &Build) -> Vec<[Vec3; 3]> {
    let verts = positions(&b.mesh.clone().into_mesh());
    let roof = b.tags.iter().filter(|t| t.0 == Tag::Roof);
    let tris = roof.flat_map(|t| verts[t.1..t.2].as_chunks::<3>().0.to_vec());
    tris.filter(|t| (t[1] - t[0]).cross(t[2] - t[0]).normalize().y.abs() > 0.2)
        .collect()
}

/// Returns the worst height, in meters, between the top of the drawn roof of `b` and where a ray cast down
/// at the same spot stops, over points spread on every roof triangle that faces up and lies above `floor`.
fn worst_miss(b: &Build, floor: f32) -> f32 {
    let (tris, shape, mut worst) = (roofing(b), solid(b), 0.0f32);
    let mix = [[4.0, 1.0, 1.0], [1.0, 4.0, 1.0], [1.0, 1.0, 4.0], [1.0; 3]];
    for (t, w) in tris.iter().flat_map(|t| mix.map(|w| (t, w))) {
        let p = (t[0] * w[0] + t[1] * w[1] + t[2] * w[2]) / (w[0] + w[1] + w[2]);
        let top = column(&tris, p.x, p.z).into_iter().fold(p.y, f32::max);
        let from = Vec3::new(p.x, top + 2.0, p.z);
        let hit = shape.cast_ray(Vec3::ZERO, Quat::IDENTITY, from, Vec3::NEG_Y, 10.0, true);
        let miss = hit.map_or(f32::MAX, |(toi, _)| (2.0 - toi).abs());
        worst = worst.max(if top < floor { 0.0 } else { miss });
    }
    worst
}

/// Every roof kind but the hip stops a ray within 5 cm of its drawn surface everywhere. The hip does within
/// a fifth of its rise, the widest gap between two of its inner steps, above the lowest of them.
#[test]
fn roofs_are_solid_where_drawn() {
    let t = Terrain::new(1);
    for roof in ROOFS {
        let mut b = built(&t, Kind::Village, 3);
        draw(&mut b, roof);
        let low = b.base_y + roof.1[1] + 0.13 * roof.3;
        let hip = roof.0 == "hip";
        let (floor, allowed) = if hip {
            (low, 0.2 * roof.3)
        } else {
            (f32::MIN, 0.05)
        };
        let miss = worst_miss(&b, floor);
        eprintln!(
            "{}: {} colliders, worst miss {miss:.3} m",
            roof.0,
            b.parts.len()
        );
        assert!(
            miss <= allowed,
            "{}: a ray stops {miss} m from the roof",
            roof.0
        );
    }
}

/// Asserts that no point of the room of `house`, inside its walls and under their tops, is inside a collider
/// of the roof-only build `b`.
fn check_room(b: &Build, house: House, what: &str) {
    let (s, shape, turn) = (house.shell(), solid(b), Quat::from_rotation_y(b.yaw));
    let inner = s.inner();
    for (i, j, k) in
        (0..=8).flat_map(|i| (0..=8).flat_map(move |j| (0..=6).map(move |k| (i, j, k))))
    {
        let local = Vec3::new(
            inner.x * (i as f32 / 4.0 - 1.0),
            0.05 + (s.h - 0.06) * k as f32 / 6.0,
            inner.y * (j as f32 / 4.0 - 1.0),
        );
        let p = turn * local + Vec3::Y * b.base_y;
        let inside = shape.contains_point(Vec3::ZERO, Quat::IDENTITY, p);
        assert!(!inside, "{what}: the roof fills the room at {local}");
    }
}

/// In every village style, house kind, pitch and choice of hipped or gabled roof, the roof colliders stay
/// out of the room under them.
#[test]
fn rooms_stay_free_under_roofs() {
    let t = Terrain::new(1);
    let biomes = [
        Biome::Meadow,
        Biome::Taiga,
        Biome::Desert,
        Biome::Tropical,
        Biome::Cherry,
    ];
    for (biome, house) in biomes.into_iter().flat_map(|b| ALL_HOUSES.map(|h| (b, h))) {
        let s = house.shell();
        for (rise, pick) in [(0.9, 0.0), (1.6, 0.0), (1.6, 0.9), (2.2, 0.9)] {
            let mut b = built(&t, Kind::Village, 3);
            b.cap_roof(&style_for(biome, 3), s.tops(s.h), rise, pick);
            check_room(&b, house, &format!("{house:?} in {biome:?} at {rise}"));
        }
    }
    let s = House::Shed.shell();
    let mut b = built(&t, Kind::Village, 3);
    let roof = (
        Vec3::new(-s.hx - 0.3, s.h, -s.hz - 0.3),
        Vec3::new(s.hx + 0.3, s.h, s.hz + 0.4),
    );
    b.lean_over(s.tops(s.h), roof, 0.7, (Mat::deco(GREY), GREY));
    check_room(&b, House::Shed, "shed");
}
