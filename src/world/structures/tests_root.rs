//! Tests that the site of a cell is remembered by the cell counted from the middle of its face: every
//! view of the face, wherever it is rooted, finds the same structure at the same place of the planet.

use bevy::math::DVec2;

use super::globes::{faces, villages};
use super::site::{CELL, structure_in_cell};
use crate::world::Terrain;
use crate::world::globe::BLOCK;

/// Returns `face` rooted `blocks` whole blocks from where it is rooted.
fn rooted(face: &Terrain, blocks: DVec2) -> Terrain {
    let chart = face.chart().rooted(face.chart().origin() + blocks * BLOCK);
    Terrain::on(face.globe().clone(), chart)
}

/// A village found through the chart rooted in the middle of its face is found again through a chart
/// rooted blocks away, in the cell the same whole cells from the middle: the same kind, turn and look,
/// within two millimetres of the same place.
#[test]
fn a_site_is_the_same_through_every_root() {
    let mut seen = 0;
    for face in faces(42, 6000.0) {
        let other = rooted(&face, DVec2::new(1.0, -1.0));
        let by = other.shift(CELL) - face.shift(CELL);
        for site in villages(&face, 3) {
            let cell = (site.origin / CELL).floor().as_ivec2();
            assert_eq!(structure_in_cell(&face, cell), Some(site));
            let again = structure_in_cell(&other, cell - by).expect("the same village");
            let was = face.embed(site.origin.x, site.floor, site.origin.y);
            let is = other.embed(again.origin.x, again.floor, again.origin.y);
            assert!(was.distance(is) < 2.0e-3, "{}", was.distance(is));
            assert_eq!(
                (again.kind, again.yaw, again.variant),
                (site.kind, site.yaw, site.variant)
            );
            seen += 1;
        }
    }
    assert!(seen > 0, "no village to compare");
}
