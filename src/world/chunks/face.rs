//! Which cube face the shared terrain view shows: the one under the focus.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::world::Terrain;
use crate::world::focus::Focus;
use crate::world::globe::{Chart, Face};

/// How far past the edge of the current face the focus goes before the view changes face, in
/// face units. It keeps a focus that walks along an edge from swapping the view every frame.
const MARGIN: f64 = 0.004;

/// Returns the view of the face that holds chart position `at` of `terrain`, and `at` in it.
pub fn rehome(terrain: &Terrain, at: Vec3) -> (Terrain, Vec3) {
    let pos = terrain.embed(at.x, at.y, at.z);
    let face = Face::locate(pos.normalize()).0;
    let globe = terrain.globe();
    let view = Terrain::on(globe.clone(), globe.face_chart(face));
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

/// Replaces the terrain view by the chart of the face under the focus once it leaves the face.
pub fn follow_face(mut commands: Commands, focus: Focus, terrain: Res<Terrain>) {
    let Some(dir) = focus.planet().map(|p| p.normalize_or(DVec3::Y)) else {
        return;
    };
    if let Some(face) = next_face(terrain.chart(), dir) {
        let globe = terrain.globe();
        commands.insert_resource(Terrain::on(globe.clone(), globe.face_chart(face)));
    }
}
