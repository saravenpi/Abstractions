//! Tests of the ways structures reach the planet: the same site from two chunks, the chart output against
//! the rigid output, and normals and lights that turn with the ground.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::audit::positions;
use super::brush::Build;
use super::carry::Target;
use super::globes::{RADII, faces, normals, planet, villages};
use super::site::{Site, structure_in_cell};
use super::{DRAW, Terrain, build_chunk, build_in};
use crate::world::CHUNK_SIZE;
use crate::world::mesh::Frame;
use crate::world::scatter::chunk_center;

/// Returns the first `want` structures of the chart `face` among the cells around its middle.
fn sites(face: &Terrain, want: usize) -> Vec<Site> {
    let cells = (-9..9).flat_map(|x| (-9..9).map(move |y| IVec2::new(x, y)));
    let found = cells.filter_map(|c| structure_in_cell(face, c));
    found.take(want).collect()
}

/// Returns the chunk that holds the origin of `site`.
fn chunk_of(site: &Site) -> IVec2 {
    (site.origin / CHUNK_SIZE).floor().as_ivec2()
}

/// Draws `site` as chunk `coord` of `face` would, and returns its vertices and lights as planet positions.
fn drawn(face: &Terrain, site: Site, coord: IVec2) -> Vec<DVec3> {
    let view = face.around(site.origin.x, site.origin.y);
    let frame = Frame::at(face, chunk_center(coord));
    let mut b = Build::on((face, &view), chunk_center(coord), site);
    DRAW[site.kind as usize](&mut b);
    let out = b.finish(Target::Frame(frame));
    let lights = out.lights.iter().map(|l| l.pos);
    let all = positions(&out.mesh).into_iter().chain(lights);
    all.map(|p| planet(&frame, p)).collect()
}

/// On globes of 3 and 40 km: a site drawn from its own chunk and from the next chunk is the same on the
/// planet within 1 mm, vertex by vertex and light by light.
#[test]
fn a_site_is_the_same_from_two_chunks() {
    for radius in RADII {
        let face = &faces(42, radius)[2];
        let mut all = sites(face, 8);
        all.extend(villages(face, 1));
        assert!(all.len() >= 6, "{} sites", all.len());
        for site in all {
            let own = chunk_of(&site);
            let (a, b) = (
                drawn(face, site, own),
                drawn(face, site, own + IVec2::new(1, -1)),
            );
            assert_eq!(a.len(), b.len());
            let off = a
                .iter()
                .zip(&b)
                .map(|(p, q)| p.distance(*q))
                .fold(0.0, f64::max);
            assert!(off < 1e-3, "{site:?} differs by {off} m between two chunks");
        }
    }
}

/// On the 3 km globe: the chart output of `build_chunk`, warped point by point the way `chunks::embed` does,
/// lands within 2 mm of the rigid output of `build_in`, with the same normals, lights and collider.
#[test]
fn chart_output_warps_onto_the_rigid_output() {
    let face = &faces(42, RADII[0])[2];
    let mut all = sites(face, 6);
    all.extend(villages(face, 1));
    for site in all {
        let (coord, center) = (chunk_of(&site), chunk_center(chunk_of(&site)));
        let frame = Frame::at(face, center);
        let warp = |p: Vec3| frame.local(face.embed(center.x + p.x, p.y, center.y + p.z));
        let (Some(old), Some(new)) = (build_chunk(face, coord), build_in(face, coord, &frame))
        else {
            panic!("{site:?} builds nothing");
        };
        let (from, to) = (positions(&old.mesh), positions(&new.mesh));
        assert_eq!(from.len(), to.len());
        let off = from.iter().zip(&to).map(|(p, q)| warp(*p).distance(*q));
        let worst = off.fold(0.0, f32::max);
        assert!(
            worst < 2e-3,
            "{site:?}: chart output lands {worst} m from the rigid output"
        );
        let lights = old.lights.iter().zip(&new.lights);
        assert!(
            lights
                .clone()
                .all(|(a, b)| warp(a.pos).distance(b.pos) < 2e-3 && a.kind == b.kind)
        );
        assert_eq!(old.lights.len(), new.lights.len());
        assert_eq!(old.collider.is_some(), new.collider.is_some());
        assert!(new.rigid && !old.rigid);
    }
}

/// On globes of 3 and 40 km, in the rigid output of `build_in`: every stored normal is the normal of its
/// carried triangle, and every light stands upright along the local up where it is.
#[test]
fn rigid_output_turns_normals_and_lights() {
    for radius in RADII {
        let face = &faces(42, radius)[2];
        let mut all = sites(face, 6);
        all.extend(villages(face, 1));
        for site in all {
            let frame = Frame::at(face, chunk_center(chunk_of(&site)));
            let Some(out) = build_in(face, chunk_of(&site), &frame) else {
                panic!("{site:?} builds nothing");
            };
            let (at, stored) = (positions(&out.mesh), normals(&out.mesh));
            let tris = at.as_chunks::<3>().0.iter().zip(stored.as_chunks::<3>().0);
            let real = |t: &[Vec3; 3]| (t[1] - t[0]).cross(t[2] - t[0]);
            let bent = |(t, n): (&[Vec3; 3], &[Vec3; 3])| {
                real(t).length() > 1e-4 && real(t).normalize().dot(n[0]) < 0.999
            };
            assert_eq!(
                tris.filter(|pair| bent(*pair)).count(),
                0,
                "{site:?}: normals left behind"
            );
            let up = |p: Vec3| frame.turn(planet(&frame, p).normalize());
            let upright = out
                .lights
                .iter()
                .all(|l| (l.rot * Vec3::Y).dot(up(l.pos)) > 0.999_99);
            assert!(upright, "{site:?}: a light leans");
        }
    }
}
