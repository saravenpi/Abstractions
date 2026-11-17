//! Tests that every door frame fits the wall opening it stands in.

use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};

use bevy::prelude::*;

use super::coast::BASE;
use super::measure::fights;
use super::probe::{Bx, Shot};
use super::site::Kind;
use super::tags::Tag;
use super::tests::built;
use super::village::{ALL_HOUSES, House};
use super::{DRAW, Terrain};
use crate::world::Biome;

/// Every biome that has its own village style.
pub const BIOMES: [Biome; 5] = [
    Biome::Meadow,
    Biome::Taiga,
    Biome::Desert,
    Biome::Tropical,
    Biome::Cherry,
];

/// Yaw `House::draw_variant` gives its lot.
pub const YAW: f32 = 0.3;

/// Yaw and floor height of the site `built` makes.
pub const SITE: (f32, f32) = (0.7, 5.0);

/// Draws every house kind in every village style for `variants` lot variations and measures each.
pub fn every_house(variants: u32, mut check: impl FnMut(&Shot, House, &str)) {
    let t = Terrain::new(1);
    for biome in BIOMES {
        for house in ALL_HOUSES {
            for v in 0..variants {
                let mut b = built(&t, Kind::Village, v * 7919 + 3);
                house.draw_variant(&mut b, biome, v * 7919 + 3);
                let what = format!("{house:?} in {biome:?} #{v}");
                check(&Shot::of(&b, YAW, SITE.1), house, &what);
            }
        }
    }
}

/// Returns the ground floor doorway of the walls of `shot`: the hole under the lintel, as a box through the wall.
fn opening(shot: &Shot) -> Option<Bx> {
    let walls = shot.boxes(Tag::Wall);
    let lintel = walls.iter().find(|w| w.0.y > 1.0 && w.1.y < 3.6)?;
    let sill = |w: &&Bx| w.0.y.abs() < 0.01 && (w.0.z - lintel.0.z).abs() < 0.01;
    let sides = walls.iter().filter(sill);
    let flank =
        sides.filter(|w| (w.1.x - lintel.0.x).abs() < 0.01 || (w.0.x - lintel.1.x).abs() < 0.01);
    assert_eq!(flank.count(), 2, "lintel without two wall ends");
    Some((
        Vec3::new(lintel.0.x, 0.0, lintel.0.z),
        Vec3::new(lintel.1.x, lintel.0.y, lintel.1.z),
    ))
}

/// Returns the top of the floor piece under the middle of the doorway `hole`.
fn threshold(shot: &Shot, hole: &Bx) -> f32 {
    let mid = (hole.0 + hole.1) * 0.5;
    let under =
        |(_, b): &(usize, Bx)| b.0.x < mid.x && b.1.x > mid.x && b.0.z < mid.z && b.1.z > mid.z;
    let tops = shot.closed().into_iter().filter(under).map(|(_, b)| b.1.y);
    tops.filter(|y| *y < 0.3).fold(f32::MIN, f32::max)
}

/// Returns how far, in meters, the frame of `shot` misses the doorway `hole`: the worst of its inner sides and top
/// against the hole, its foot against the threshold, and how it wraps both wall faces and covers the reveal.
fn misfit(shot: &Shot, hole: &Bx) -> f32 {
    let mut frame = shot.boxes(Tag::Frame);
    assert_eq!(frame.len(), 3, "a frame is two jambs and a head");
    frame.sort_by(|a, b| (a.1.x - a.0.x).total_cmp(&(b.1.x - b.0.x)));
    let (head, mut jambs) = (frame[2], [frame[0], frame[1]]);
    jambs.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));
    let floor = threshold(shot, hole);
    let inner = [
        jambs[0].1.x - hole.0.x,
        hole.1.x - jambs[1].0.x,
        hole.1.y - head.0.y,
    ];
    let proud = inner.iter().map(|d| (d - 0.007).abs() - 0.003);
    let wrap = frame
        .iter()
        .flat_map(|f| [hole.0.z - f.0.z, f.1.z - hole.1.z]);
    let faces = wrap.map(|d| (d - 0.05).abs() - 0.046);
    let foot = jambs.iter().map(|j| (j.0.y - floor).abs() - 0.01);
    let joined = jambs.iter().map(|j| (head.0.y - j.1.y).abs() - 0.01);
    let ends = [jambs[0].0.x - head.0.x, head.1.x - jambs[1].1.x].map(|d: f32| d.abs() - 0.01);
    let all = proud.chain(faces).chain(foot).chain(joined).chain(ends);
    all.fold(f32::MIN, f32::max)
}

/// Asserts that the frame of `shot` exists only around a doorway, fits it within a centimeter and fights no face.
fn check_frame(shot: &Shot, what: &str) {
    let Some(hole) = opening(shot) else {
        assert!(
            shot.boxes(Tag::Frame).is_empty(),
            "{what}: a door frame stands without a wall"
        );
        return;
    };
    let off = misfit(shot, &hole);
    assert!(
        off <= 1e-3,
        "{what}: the frame misses its doorway by {off} m"
    );
    let frame = |t: usize| shot.is(Tag::Frame, t * 3);
    let clash = fights(&shot.verts)
        .into_iter()
        .filter(|(a, b)| frame(*a) || frame(*b));
    assert_eq!(
        clash.count(),
        0,
        "{what}: frame faces lie in the plane of other faces"
    );
}

/// The lighthouse door lies flat on one of the eight faces of the tower: its back is sunk in the wall and
/// its front stands out of it, at every corner.
#[test]
fn the_lighthouse_door_lies_on_a_wall_face() {
    let t = Terrain::new(1);
    for variant in 0..8 {
        let mut b = built(&t, Kind::Lighthouse, variant);
        b.site.yaw = variant as f32 * 0.83;
        b.yaw = b.site.yaw;
        DRAW[Kind::Lighthouse as usize](&mut b);
        let shot = Shot::of(&b, 0.0, 0.0);
        let (start, _) = shot.closed()[0];
        let reach = |p: &Vec3| {
            let faces = (0..8).map(|k| Vec2::from_angle((k as f32 + 0.5) * FRAC_PI_4));
            faces.map(|n| n.dot(p.xz())).fold(f32::MIN, f32::max) - BASE * FRAC_PI_8.cos()
        };
        let out: Vec<f32> = shot.verts[start..start + 36].iter().map(reach).collect();
        let (back, front) = (
            out.iter().copied().fold(f32::MAX, f32::min),
            out.iter().copied().fold(f32::MIN, f32::max),
        );
        assert!(
            back < -0.02 && front > 0.02,
            "yaw {}: door from {back} to {front} m off the wall",
            b.site.yaw
        );
        let level = out
            .iter()
            .all(|d| (d - back).abs() < 1e-3 || (d - front).abs() < 1e-3);
        assert!(level, "yaw {}: the door is skewed on its wall", b.site.yaw);
    }
}

/// Every village house and the cabin: the frame stands only in a wall, its inner sides and top are the doorway
/// within 1 cm, its foot is on the threshold, it wraps both wall faces and none of its faces fights another.
#[test]
fn door_frames_fit_their_doorways() {
    every_house(12, |shot, _, what| check_frame(shot, what));
    let t = Terrain::new(1);
    let mut b = built(&t, Kind::Cabin, 3);
    DRAW[Kind::Cabin as usize](&mut b);
    check_frame(&Shot::of(&b, SITE.0, SITE.1), "Cabin");
}
