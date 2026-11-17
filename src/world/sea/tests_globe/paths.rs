//! Places and paths on the test planet: its shore, its cube edges and its terrain mesh.

use std::sync::Arc;

use bevy::math::DVec3;

use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Sea level radius of the test planet in meters.
pub const RADIUS: f64 = 3000.0;

/// Returns the view of `face` of `globe`.
pub fn face_view(globe: &Arc<Globe>, face: Face) -> Terrain {
    Terrain::on(globe.clone(), globe.face_chart(face))
}

/// Returns the height the finest terrain mesh draws in direction `dir`, on the face that holds it.
pub fn mesh_height(globe: &Arc<Globe>, dir: DVec3) -> f32 {
    let view = face_view(globe, Face::locate(dir).0);
    let at = view.chart().locate(dir);
    view.mesh_height(at.x, at.y)
}

/// Returns the point `reach` meters from direction `from` along the great circle of `bearing`.
pub fn along(from: DVec3, bearing: f64, reach: f64) -> DVec3 {
    let east = DVec3::Y.cross(from).normalize();
    let side = east * bearing.cos() + from.cross(east) * bearing.sin();
    let angle = reach / RADIUS;
    (from * angle.cos() + side * angle.sin()) * RADIUS
}

/// Returns a point of the shore of `globe`: the first place east of `from` where the ground
/// goes under the sea.
pub fn shore_east_of(globe: &Arc<Globe>, from: DVec3) -> DVec3 {
    let dry = |reach: f64| globe.height(along(from, 0.0, reach).normalize()) > 0.0;
    let reach = (1..20_000).map(f64::from).find(|reach| !dry(*reach));
    along(from, 0.0, reach.unwrap_or_default())
}

/// Returns the test planet and a point of its shore near the spawn.
pub fn shore() -> (Arc<Globe>, DVec3) {
    let globe = Arc::new(Globe::new(1, RADIUS, 22.0));
    let north = face_view(&globe, Face::PosY);
    let spawn = north.find_spawn();
    let at = shore_east_of(&globe, north.embed(spawn.x, 0.0, spawn.z).normalize());
    (globe, at)
}

/// Returns a point of the shore that lies on the edge between the north face and another face.
pub fn shore_on_an_edge(globe: &Arc<Globe>) -> Option<DVec3> {
    let corners = [DVec3::X, DVec3::Z, DVec3::NEG_X, DVec3::NEG_Z];
    let edges = corners.iter().flat_map(|side| {
        let along = DVec3::Y.cross(*side);
        (-900..900).map(move |i| (DVec3::Y + *side + along * (f64::from(i) * 0.001)).normalize())
    });
    let mut last = None;
    for dir in edges {
        let height = globe.height(dir);
        if last.is_some_and(|before: f32| before * height < 0.0 && before.abs() < 0.5) {
            return Some(dir * RADIUS);
        }
        last = Some(height);
    }
    None
}
