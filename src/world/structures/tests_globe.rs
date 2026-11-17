//! Tests of village houses on true spheres: they keep the shape they have on flat land and stand on their
//! ground.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Terrain;
use super::audit::{Piece, positions};
use super::brush::Build;
use super::carry::Target;
use super::globes::{RADII, faces, planet, villages};
use super::site::{Kind, Site};
use super::tags::Tag;
use super::village::{self, Lot};
use crate::world::globe::Chart;
use crate::world::mesh::Frame;

/// Lowest and highest allowed height of the top corners of a platform above the ground under them. The
/// layout samples the ground a little outside the corners, so a bank can stand a few centimeters over one.
const TOP: (f32, f32) = (-0.3, 3.6);

/// Highest allowed height of the bottom corners of a masonry platform above the ground under them.
const BOTTOM: f32 = -0.3;

/// How far, in meters, the top of a platform may sit from the floor height its lot chose.
const LEVEL: f32 = 0.02;

/// The planet positions of the first platform and of the walls of one drawn house.
struct Drawn {
    plinth: Vec<DVec3>,
    walls: Vec<DVec3>,
}

/// Draws the house of `lot` of the village `site` on `charts`, the chart of the chunk then the view of the
/// village, carried rigidly to the planet.
fn draw(charts: (&Terrain, &Terrain), site: Site, lot: &Lot) -> Drawn {
    let frame = Frame::at(charts.1, lot.pos);
    let mut b = Build::on(charts, site.origin, site);
    village::draw_lot(&mut b, lot);
    let boxes = b.audit.iter().filter(|a| a.0 == Piece::Closed);
    let plinth = boxes.map(|a| (a.1, a.2)).next().unwrap_or((0, 0));
    let wall = b
        .tags
        .iter()
        .find(|t| t.0 == Tag::Wall)
        .map_or((0, 0), |t| (t.1, t.2));
    let verts = positions(&b.finish(Target::Frame(frame)).mesh);
    let part = |r: (usize, usize)| verts[r.0..r.1].iter().map(|p| planet(&frame, *p)).collect();
    Drawn {
        plinth: part(plinth),
        walls: part(wall),
    }
}

/// Returns the largest difference, in meters, between the distances from five wall vertices to every wall
/// vertex of the house `a` and of the same house `b`: zero when their walls have the same shape and size.
fn misshape(a: &[DVec3], b: &[DVec3]) -> f64 {
    assert_eq!(a.len(), b.len(), "wall vertex counts");
    let last = a.len().saturating_sub(1);
    let marks = [0, last / 4, last / 2, last * 3 / 4, last];
    let off = |i: usize, m: usize| (a[i].distance(a[m]) - b[i].distance(b[m])).abs();
    let all = (0..a.len()).flat_map(|i| marks.map(|m| off(i, m)));
    all.fold(0.0, f64::max)
}

/// Returns, for the platform `plinth` of a house on `view`, the lowest and highest height of its top corners
/// above the ground under each, and the highest height of its bottom corners above the ground under each.
fn standing(view: &Terrain, plinth: &[DVec3], floor: f32) -> (f32, f32, f32) {
    let at = |p: &DVec3| view.locate(*p);
    let over = |p: &DVec3| at(p).y - view.height(at(p).x, at(p).z);
    let top = plinth.iter().filter(|p| at(p).y > floor - 0.1).map(over);
    let low = plinth.iter().filter(|p| at(p).y <= floor - 0.1).map(over);
    let (lo, hi) = top.fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v), h.max(v)));
    (lo, hi, low.fold(f32::MIN, f32::max))
}

/// Asserts that the house of `lot` stands on its ground like on flat land: the top of its platform is at
/// the floor height of the lot within 2 cm, no top corner is more than 0.3 m under the ground nor 3.6 m
/// above it, and a masonry platform is buried at every bottom corner. The deck of a stilt house, 0.35 m
/// thick, stands on posts instead.
fn check_standing(view: &Terrain, house: &Drawn, lot: &Lot) {
    let (lo, hi, bottom) = standing(view, &house.plinth, lot.floor);
    assert!(
        lo >= TOP.0 && hi <= TOP.1,
        "{lot:?}: platform top {lo} to {hi} m above its ground"
    );
    let heights: Vec<f32> = house.plinth.iter().map(|p| view.locate(*p).y).collect();
    let tops = heights.iter().filter(|y| **y > lot.floor - 0.1);
    let off = tops.map(|y| (y - lot.floor).abs()).fold(0.0, f32::max);
    assert!(
        off <= LEVEL,
        "{lot:?}: platform top {off} m from its floor height"
    );
    let deck = lot.floor - heights.iter().copied().fold(f32::MAX, f32::min) < 1.0;
    assert!(
        deck || bottom <= BOTTOM,
        "{lot:?}: platform bottom {bottom} m above its ground"
    );
}

/// Returns the walls of the house of `lot` drawn on the flat world.
fn flat_walls(lot: &Lot) -> Vec<DVec3> {
    let flat = Terrain::new(1);
    let site = Site {
        kind: Kind::Village,
        origin: Vec2::ZERO,
        yaw: 0.0,
        floor: lot.floor,
        depth: 2.0,
        variant: 6,
    };
    let lot = Lot {
        pos: Vec2::ZERO,
        ..*lot
    };
    draw((&flat, &flat), site, &lot).walls
}

/// Returns the villages of the chart `face` to measure: those of the middle of the flat world, or up to six
/// of a cube face.
fn settled(face: &Terrain) -> Vec<Site> {
    let cells = (-8..8).flat_map(|x| (-8..8).map(move |y| IVec2::new(x, y)));
    match face.chart() {
        Chart::Flat => cells
            .filter_map(|c| village::village_in_cell(face, c))
            .collect(),
        _ => villages(face, 6),
    }
}

/// Measures every house of the village `site` of the chart `face`: how it stands, and how far its walls are
/// from the walls of the same house on flat land, at most `limit` meters. Returns the number of houses.
fn check_village(face: &Terrain, site: Site, limit: f64) -> usize {
    let view = face.around(site.origin.x, site.origin.y);
    let lots = village::plots(&view, site);
    for lot in &lots {
        let house = draw((face, &view), site, lot);
        check_standing(&view, &house, lot);
        let off = misshape(&house.walls, &flat_walls(lot));
        assert!(
            off <= limit,
            "{lot:?}: walls differ from flat land by {off} m"
        );
    }
    lots.len()
}

/// On flat land and on globes of 3 and 40 km, over at least 100 houses each: every platform sits at the
/// floor height of its lot and stands on its ground within the same bounds, and on the globes the walls have
/// the height and width they have on flat land within 1 mm.
#[test]
fn houses_keep_their_shape_and_ground_on_globes() {
    let mut worlds = vec![(vec![Terrain::new(1)], 1e-4)];
    worlds.extend(RADII.map(|radius| (faces(42, radius), 1e-3)));
    for (charts, limit) in worlds {
        let sites = charts
            .iter()
            .flat_map(|face| settled(face).into_iter().map(move |s| (face, s)));
        let seen: usize = sites
            .map(|(face, site)| check_village(face, site, limit))
            .sum();
        assert!(seen >= 100, "{seen} houses measured");
    }
}
