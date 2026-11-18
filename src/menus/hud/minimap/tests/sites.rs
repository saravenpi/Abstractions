//! The structures on the map: how one is stamped on the image, and which ones a patch lists.

use std::sync::Arc;
use std::time::Instant;

use bevy::math::{DVec3, IVec2, Vec2};

use crate::menus::hud::minimap::paint::{BYTES, paint, render, sites, stamp};
use crate::menus::hud::minimap::patch::{Patch, TEXELS};
use crate::world::globe::Face;
use crate::world::structures::{CELL, structure_in_cell};
use crate::world::{Globe, Terrain};

/// Radius in metres of the planet the tests stand on.
const RADIUS: f64 = 6000.0;

/// A structure is a white texel in a black box and a village a larger one, and a stamp near
/// the edge of the image stays inside it.
#[test]
fn structures_are_stamped_as_boxed_dots() {
    let patch = Patch::around(DVec3::X, 6000.0, 250.0);
    let mut pixels = vec![128; TEXELS * TEXELS * BYTES];
    let texel = 1000.0 / TEXELS as f32;
    stamp(&mut pixels, &patch, Vec2::new(0.5, -0.5) * texel, false);
    stamp(&mut pixels, &patch, Vec2::new(-200.0, 200.0), true);
    stamp(&mut pixels, &patch, Vec2::new(499.0, 499.0), true);
    let grey = |x: usize, y: usize| pixels[(y * TEXELS + x) * BYTES];
    let middle = TEXELS / 2;
    assert_eq!((grey(middle, middle), grey(middle + 1, middle)), (255, 0));
    assert_eq!(
        (grey(middle - 1, middle - 1), grey(middle + 2, middle)),
        (0, 128)
    );
    let (x, y) = (middle - 20, middle - 20);
    assert_eq!(
        (grey(x, y), grey(x + 1, y + 1), grey(x + 2, y)),
        (255, 255, 0)
    );
    assert_eq!((grey(x - 2, y - 2), grey(x + 3, y)), (0, 128));
    assert_eq!(grey(TEXELS - 1, 0), 255);
    assert!(
        pixels
            .as_chunks::<BYTES>()
            .0
            .iter()
            .all(|texel| texel[3] == 128)
    );
}

/// Returns the chart position of the first structure found on the +X face of `globe`.
fn a_site(globe: &Arc<Globe>) -> (Terrain, Vec2, f32) {
    let face = Terrain::on(globe.clone(), globe.face_chart(Face::PosX));
    let cells = (-12..12).flat_map(|x| (-12..12).map(move |y| IVec2::new(x, y)));
    let site = cells
        .filter_map(|cell| structure_in_cell(&face, cell))
        .next();
    let site = site.expect("a structure within two kilometres of the middle of the face");
    assert!(site.origin.abs().max_element() < 12.0 * CELL);
    (face, site.origin, site.floor)
}

/// A patch lists the structures that stand on it, each where it stands, and the finished
/// image carries their stamps on the nearest range only.
#[test]
fn a_patch_lists_the_structures_on_it() {
    let globe = Arc::new(Globe::new(7, RADIUS, 22.0));
    let (face, origin, floor) = a_site(&globe);
    let pos = face.embed(origin.x, floor, origin.y);
    let patch = Patch::around(pos.normalize(), RADIUS, 250.0);
    let started = Instant::now();
    let found = sites(&globe, &patch);
    println!(
        "minimap sites: {} on a 250 m patch in {:?}",
        found.len(),
        started.elapsed()
    );
    assert!(found.iter().any(|(at, _)| at.length() < 1.0), "{found:?}");
    assert!(
        found
            .iter()
            .all(|(at, _)| at.abs().max_element() < patch.half())
    );
    let (plain, stamped) = (paint(&globe, &patch), render(&globe, &patch));
    assert_eq!(plain.len(), stamped.len());
    assert!(plain != stamped);
    let wide = Patch::around(pos.normalize(), RADIUS, 800.0);
    assert!(paint(&globe, &wide) == render(&globe, &wide));
}
