//! Globe tests: chart round trips, face handedness, edge adjacency and the flat legacy view.

use std::sync::Arc;

use bevy::math::{DVec2, DVec3};

use super::*;
use crate::world::Terrain;

mod coarse;
mod cost;
mod frozen;
mod seams;
mod sphere;

/// Returns evenly spread coordinates strictly inside -1 to 1.
fn spread() -> impl Iterator<Item = f64> {
    (0..=20).map(|i| -0.99 + 0.099 * f64::from(i))
}

/// A direction goes to its face chart and back within 1e-9 on all six faces.
#[test]
fn face_charts_round_trip() {
    for face in Face::ALL {
        let [n, a, b] = face.axes();
        assert!((b.cross(a) - n).length() < 1e-12, "{face:?} is left-handed");
        for (u, v) in spread().flat_map(|u| spread().map(move |v| (u, v))) {
            let dir = face.dir(u, v);
            let (found, fu, fv) = Face::locate(dir);
            assert_eq!(found, face);
            assert!((fu - u).abs() < 1e-9 && (fv - v).abs() < 1e-9);
            assert!((found.dir(fu, fv) - dir).length() < 1e-9);
            let (pu, pv) = face.project(face.dir(u * 1.5, v));
            assert!((pu - u * 1.5).abs() < 1e-9 && (pv - v).abs() < 1e-9);
        }
    }
}

/// A direction goes to a tangent chart and back within 1e-9, and distances from the origin are true.
#[test]
fn tangent_charts_round_trip() {
    let radius = 6000.0;
    for face in Face::ALL {
        let origin = face.dir(0.4, -0.7);
        let offset = DVec2::new(310.0, -95.0);
        let chart = Tangent::new(origin, DVec3::new(0.3, 0.5, 0.8), radius, offset);
        assert!((chart.dir(offset) - origin).length() < 1e-12);
        assert!((chart.north.cross(chart.east) + origin).length() < 1e-12);
        for (u, v) in spread().flat_map(|u| spread().map(move |v| (u, v))) {
            let dir = face.dir(0.4 + 0.2 * u, -0.7 + 0.2 * v);
            let p = chart.locate(dir);
            assert!((chart.dir(p) - dir).length() < 1e-9);
            let arc = dir.angle_between(origin) * radius;
            assert!(((p - offset).length() - arc).abs() < 1e-6);
        }
    }
}

/// Returns the directions of the two ends of the edge of `id` crossed by the step `(dx, dy)`.
fn edge_ends(id: ChunkId, dx: i32, dy: i32) -> [DVec3; 2] {
    let side = |d: i32| f64::from((d + 1) / 2);
    if dx != 0 {
        [id.dir(side(dx), 0.0), id.dir(side(dx), 1.0)]
    } else {
        [id.dir(0.0, side(dy)), id.dir(1.0, side(dy))]
    }
}

/// Returns true when `dir` is a corner of `id`.
fn is_corner(id: ChunkId, dir: DVec3) -> bool {
    [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
        .iter()
        .any(|&(s, t)| (id.dir(s, t) - dir).length() < 1e-9)
}

/// Every node shares a whole edge with each of its four neighbours, also across face edges.
#[test]
fn neighbors_share_edges() {
    const STEPS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let level = 3;
    let mut crossings = 0;
    for face in Face::ALL {
        for (x, y) in (0..8).flat_map(|x| (0..8).map(move |y| (x, y))) {
            let id = ChunkId { face, level, x, y };
            for (dx, dy) in STEPS {
                let next = id.neighbor(dx, dy).unwrap();
                assert_ne!(next, id);
                crossings += usize::from(next.face != face);
                let ends = edge_ends(id, dx, dy);
                assert!(ends.iter().all(|&e| is_corner(next, e)), "{id:?} {next:?}");
                let back = STEPS.iter().any(|&(a, b)| next.neighbor(a, b) == Some(id));
                assert!(back, "{next:?} does not lead back to {id:?}");
            }
        }
    }
    assert_eq!(crossings, 6 * 4 * 8);
    let corner = ChunkId {
        face: Face::PosY,
        level,
        x: 0,
        y: 7,
    };
    assert_eq!(corner.neighbor(-1, 1), None);
    assert_eq!(corner.neighbor(1, 1).map(|n| n.face), Some(Face::PosZ));
}

/// The legacy terrain is the flat chart of a 6 km globe and reads the plane unchanged.
#[test]
fn flat_terrain_reads_the_plane() {
    let t = Terrain::legacy(5);
    assert_eq!(t.chart(), Chart::Flat);
    assert_eq!(t.globe().radius(), 6000.0);
    assert_eq!(
        t.height(130.5, -77.25),
        t.globe().height_flat(130.5, -77.25)
    );
    assert_eq!(t.embed(1.0, 2.0, 3.0), DVec3::new(1.0, 2.0, 3.0));
    assert_eq!(
        t.locate(DVec3::new(1.0, 2.0, 3.0)).to_array(),
        [1.0, 2.0, 3.0]
    );
    assert_eq!(t.up(40.0, 9.0), DVec3::Y);
    assert_eq!(t.around(40.0, 9.0).chart(), Chart::Flat);
    assert_eq!(t.globe().leaf_level(), 7);
    let scale = 4096.0;
    let face = Face::PosY;
    let origin = bevy::math::DVec2::ZERO;
    let middle = Chart::Face {
        face,
        scale,
        origin,
    };
    assert_eq!(t.globe().face_chart(face), middle);
}

/// Face and tangent views embed and locate consistently, and a tangent view keeps the site and its axes.
#[test]
fn views_embed_and_locate() {
    let globe = Arc::new(Globe::new(9, 6000.0, 22.0));
    for face in Face::ALL {
        let t = Terrain::on(globe.clone(), globe.face_chart(face));
        let (x, z) = (1800.0, -2600.0);
        let p = t.embed(x, 12.5, z);
        assert!((p.length() - 6012.5).abs() < 1e-9);
        assert!((t.locate(p) - bevy::math::Vec3::new(x, 12.5, z)).length() < 1e-2);
        assert_eq!(t.height(x, z), globe.height(t.up(x, z)));
        let site = t.around(x, z);
        assert!((site.embed(x, 12.5, z) - p).length() < 1e-6);
        assert!((site.locate(p) - bevy::math::Vec3::new(x, 12.5, z)).length() < 1e-2);
        assert!(site.chart().east(x, z).dot(t.chart().east(x, z)) > 0.999_999);
        let step = site.embed(x + 30.0, 0.0, z + 40.0) - site.embed(x, 0.0, z);
        assert!((step.length() - 50.0).abs() < 1e-2);
        assert!((site.height(x, z) - t.height(x, z)).abs() < 1e-3);
    }
}
