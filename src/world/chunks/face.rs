//! Which cube face the shared terrain view shows, and where it is rooted: under the focus.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::world::Terrain;
use crate::world::focus::Focus;
use crate::world::globe::{Chart, Face};

/// How far past the edge of the current face the focus goes before the view changes face, in
/// face units. It keeps a focus that walks along an edge from swapping the view every frame.
const MARGIN: f64 = 0.004;

/// Returns the view of the face that holds chart position `at` of `terrain`, rooted near it, and `at` in it.
pub fn rehome(terrain: &Terrain, at: Vec3) -> (Terrain, Vec3) {
    let pos = terrain.embed(at.x, at.y, at.z);
    let view = Terrain::near(terrain.globe(), pos);
    let at = view.locate(pos);
    (view, at)
}

/// Returns the face a view through `chart` should move to for a focus in direction `dir`, or
/// nothing while the focus is on the face of the chart or within the margin past its edge.
pub fn next_face(chart: Chart, dir: DVec3) -> Option<Face> {
    let holder = Face::locate(dir).0;
    let Chart::Face { face, .. } = chart else {
        return Some(holder);
    };
    if face == holder {
        return None;
    }
    let (u, v) = face.project(dir);
    let behind = dir.dot(face.axes()[0]) <= 0.0;
    (behind || u.abs().max(v.abs()) > 1.0 + MARGIN).then_some(holder)
}

/// Keeps the terrain view under the focus: replaces it by the chart of the face under the focus once it
/// leaves the face, and roots it again near the focus once that is more than a block from its origin. The
/// anchor is rebased long before, so the origin of the chart follows the anchor and chart positions near
/// the focus stay small on a planet of any size.
pub fn follow_face(mut commands: Commands, focus: Focus, terrain: Res<Terrain>) {
    let Some(dir) = focus.planet().map(|p| p.normalize_or(DVec3::Y)) else {
        return;
    };
    let globe = terrain.globe();
    if let Some(face) = next_face(terrain.chart(), dir) {
        let chart = globe.face_chart(face).near(dir);
        commands.insert_resource(Terrain::on(globe.clone(), chart));
    } else if let Some(view) = terrain.followed(dir) {
        commands.insert_resource(view);
    }
}
