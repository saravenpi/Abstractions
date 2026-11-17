//! Tests that roofs rest on what carries them: walls reach up to the roof, posts end at it and a roof layer
//! that lies on another touches it.

use bevy::prelude::*;

use super::camp::{Layout, draw_layout};
use super::gaps::{climb, column};
use super::probe::{Bx, Shot};
use super::site::Kind;
use super::tags::Tag;
use super::tests::built;
use super::tests_doors::{BIOMES, SITE, every_house};
use super::{DRAW, Terrain, village};

/// Largest gap, in meters, between a roof and what carries it.
const GAP: f32 = 0.02;

/// Returns the largest gap over the top of `wall` between the wall, continued by the fill above it, and the
/// lowest piece over it, measured at stations along `along`: at each one the gap is the smallest found
/// across the wall.
fn gap_along(shot: &Shot, wall: &Bx, along: Vec3) -> f32 {
    let cover = shot.tris(|v| !shot.is(Tag::Wall, v));
    let fill = shot.tris(|v| shot.is(Tag::Fill, v));
    let (size, across) = (wall.1 - wall.0, Vec3::new(along.z, 0.0, along.x));
    let gap = |p: Vec3| {
        let top = climb(&fill, p.xz(), wall.1.y);
        let over = column(&cover, p.x, p.z)
            .into_iter()
            .filter(|y| *y >= top - GAP);
        over.fold(f32::MAX, f32::min) - top
    };
    let (len, steps) = (
        size.dot(along),
        (size.dot(along) / 0.2).ceil().max(1.0) as usize,
    );
    let mut worst = 0.0f32;
    for i in 0..=steps {
        let t = len * i as f32 / steps as f32;
        let thick =
            (0..=6).map(|k| wall.0 + along * t + across * size.dot(across) * k as f32 / 6.0);
        worst = worst.max(thick.map(gap).fold(f32::MAX, f32::min));
    }
    worst
}

/// Returns the gap between the top of `wall` and what it carries: a wall is closed when, walking along it
/// one way or the other, every station has a point across it where the wall or its fill meets the roof.
fn wall_gap(shot: &Shot, wall: &Bx) -> f32 {
    gap_along(shot, wall, Vec3::X).min(gap_along(shot, wall, Vec3::Z))
}

/// Asserts that every wall top of `shot` reaches the roof or the ceiling over it.
fn check_walls(shot: &Shot, what: &str) {
    for wall in shot.boxes(Tag::Wall) {
        let gap = wall_gap(shot, &wall);
        assert!(
            gap <= GAP,
            "{what}: {gap} m of air over the wall {:?}",
            wall
        );
    }
}

/// Asserts that every post of `shot` that carries a roof ends at it, and returns the number of posts.
fn check_posts(shot: &Shot, what: &str) -> usize {
    let roof = shot.tris(|v| shot.is(Tag::Roof, v));
    let posts = shot.ranges(Tag::Post);
    for (start, end) in &posts {
        let (lo, hi) = shot.bounds(*start, *end);
        let mid = (lo + hi) * 0.5;
        let near = column(&roof, mid.x, mid.z)
            .into_iter()
            .map(|y| (y - hi.y).abs());
        let gap = near.fold(f32::MAX, f32::min);
        assert!(
            gap <= GAP,
            "{what}: the post at {mid} ends {gap} m from its roof"
        );
    }
    posts.len()
}

/// Returns, for the roof piece drawn as vertices `own`, the points just inside its lowest edges that have
/// another roof piece under them, as their heights above it, and the number of points tried.
fn eave_gaps(shot: &Shot, own: (usize, usize)) -> (Vec<f32>, usize) {
    let mine = shot.tris(|v| (own.0..own.1).contains(&v));
    let rest = shot.tris(|v| shot.is(Tag::Roof, v) && !(own.0..own.1).contains(&v));
    let low = mine.iter().flatten().map(|p| p.y).fold(f32::MAX, f32::min);
    let mut points = Vec::new();
    for t in &mine {
        for i in 0..3 {
            let (p, q, w) = (t[i], t[(i + 1) % 3], t[(i + 2) % 3]);
            let eave = p.y.max(q.y) <= low + 1e-3 && w.y > low + 1e-3;
            let spots = (1..6).map(|k| p.lerp(q, k as f32 / 6.0));
            points.extend(
                spots
                    .filter(|_| eave)
                    .map(|s| s + (w - s).normalize() * 0.01),
            );
        }
    }
    let under = |p: &Vec3| {
        let below = column(&rest, p.x, p.z)
            .into_iter()
            .filter(|y| *y <= p.y + 1e-3 && *y >= p.y - 3.0);
        below.map(|y| p.y - y).reduce(f32::min)
    };
    (points.iter().filter_map(under).collect(), points.len())
}

/// Asserts that every roof piece of `shot` whose whole eave line lies over another roof piece touches it.
fn check_layers(shot: &Shot, what: &str) {
    for own in shot.ranges(Tag::Roof) {
        let (gaps, tried) = eave_gaps(shot, own);
        let worst = gaps.iter().copied().fold(0.0, f32::max);
        let stacked = tried > 0 && gaps.len() == tried;
        assert!(
            !stacked || worst <= GAP,
            "{what}: a roof layer hangs {worst} m above the roof under it"
        );
    }
}

/// In every house kind, style and lot variation, and in the cabin and the stilt hut: the walls, continued by
/// their fill, reach the roof or ceiling within 2 cm along their whole length, and no roof layer hangs
/// above the roof it lies on.
#[test]
fn roofs_rest_on_their_walls() {
    every_house(12, |shot, _, what| {
        check_walls(shot, what);
        check_layers(shot, what);
    });
    let t = Terrain::new(1);
    for kind in [Kind::Cabin, Kind::StiltHut] {
        let mut b = built(&t, kind, 3);
        DRAW[kind as usize](&mut b);
        let shot = Shot::of(&b, SITE.0, SITE.1);
        assert!(!shot.boxes(Tag::Wall).is_empty(), "{kind:?} has no walls");
        check_walls(&shot, &format!("{kind:?}"));
    }
}

/// Every post that carries a roof ends within 2 cm of it: porches and stalls, camp shelters and the well.
#[test]
fn posts_reach_their_roofs() {
    let mut posts = 0;
    every_house(2, |shot, _, what| posts += check_posts(shot, what));
    let t = Terrain::new(1);
    for layout in Layout::ALL {
        let mut b = built(&t, Kind::Camp, 3);
        draw_layout(&mut b, layout);
        posts += check_posts(&Shot::of(&b, SITE.0, SITE.1), &format!("{layout:?}"));
    }
    for biome in BIOMES {
        let mut b = built(&t, Kind::Village, 3);
        village::draw_well(&mut b, biome);
        posts += check_posts(&Shot::of(&b, SITE.0, SITE.1), &format!("well in {biome:?}"));
    }
    assert!(posts >= 50, "{posts} posts measured");
}
