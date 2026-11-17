//! Tests of the sea on a globe: the floor map against the terrain mesh, across anchor rebases,
//! cube edges and planets.

mod paths;
mod rig;

use std::collections::HashSet;
use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use self::paths::{RADIUS, along, face_view, shore, shore_on_an_edge};
use self::rig::Rig;
use super::depth;
use super::seat::Seat;
use crate::space::{Anchor, Shift, rebased};
use crate::universe::Universe;
use crate::world::Globe;
use crate::world::globe::Face;

/// On a globe the map waterline lies within one texel of the terrain mesh waterline.
#[test]
fn the_map_waterline_is_the_mesh_waterline_on_a_globe() {
    let (globe, shore) = shore();
    let mut rig = Rig::new(&globe, shore);
    rig.fill(shore);
    let found = rig.check_waterline(shore).len();
    assert!(found >= 12, "only {found} waterline crossings");
}

/// A rebase of the anchor leaves the seat and every texel that stayed in range untouched, and
/// the waterline stays right after it and after walking on.
#[test]
fn a_rebase_leaves_the_map_alone() {
    let (globe, shore) = shore();
    let view = face_view(&globe, Face::locate(shore.normalize()).0);
    let stand = view.locate(shore);
    let old = Anchor::surface(Universe::new(1).home(), &view, stand);
    let mut rig = Rig::new(&globe, shore);
    let first = rig.fill(old.to_frame(stand));
    let before = rig.bytes();
    let walked = stand + Vec3::new(31.0, 0.0, 12.0);
    let new = rebased(&old, RADIUS, walked).expect("a rebase past the limit");
    let moved = Shift::between(&old, &new).point(walked);
    assert!(new.to_frame(moved).distance(old.to_frame(walked)) < 1e-3);
    let onward = new.to_frame(moved + Vec3::new(9.0, 0.0, -14.0));
    let second = rig.fill(onward);
    let after = rig.bytes();
    let kept = depth::entered(None, second)
        .into_iter()
        .filter(|t| depth::inside(first, *t));
    let same = |t: IVec2| rig.stored(&before, t) == rig.stored(&after, t);
    let kept: Vec<IVec2> = kept.collect();
    assert!(kept.len() > 10_000 && kept.into_iter().all(same));
    assert!(second != first && rig.seat.holds(&globe));
    assert!(rig.check_waterline(onward).len() >= 8);
}

/// Where the shore runs over a cube edge, the map waterline matches the terrain mesh on both
/// faces, and the map is the same whichever face the shared terrain view shows.
#[test]
fn a_cube_face_change_leaves_the_map_alone() {
    let globe = Arc::new(Globe::new(1, RADIUS, 22.0));
    let shore = shore_on_an_edge(&globe).expect("a shore on a cube edge");
    let body = Universe::new(1).home();
    let mut rig = Rig::new(&globe, shore);
    let mut maps = Vec::new();
    for face in [
        Face::PosY,
        Face::locate(along(shore.normalize(), 0.0, 60.0).normalize()).0,
    ] {
        let view = face_view(&globe, face);
        let stand = view.locate(shore);
        rig.fill(Anchor::surface(body, &view, stand).to_frame(stand));
        maps.push(rig.bytes());
    }
    assert!(maps[0] == maps[1] && rig.seat.holds(&globe));
    let faces: HashSet<Face> = rig.check_waterline(shore).into_iter().collect();
    assert!(faces.len() >= 2, "{faces:?}");
}

/// Another planet does not hold the seat, so the sea is seated again there, and a focus that
/// strays far from the middle of a seat gets a new one.
#[test]
fn the_seat_follows_the_planet_and_the_focus() {
    let (globe, shore) = shore();
    let seat = Seat::at(&globe, shore);
    let other = Arc::new(Globe::new(2, 40_000.0, 9.0));
    assert!(seat.holds(&globe) && !seat.holds(&other));
    assert!(seat.coords(shore).length() < 1e-3);
    let near = seat.coords(along(shore.normalize(), 1.0, 900.0));
    let far = seat.coords(along(shore.normalize(), 1.0, 1100.0));
    assert!((near.length() - 900.0).abs() < 0.01 && !seat.strayed(near) && seat.strayed(far));
    let sited = seat.sited();
    assert!(sited.pos.distance(shore.normalize() * RADIUS) < 1e-6);
    assert!((sited.rot * DVec3::Y).distance(shore.normalize()) < 1e-9);
}
