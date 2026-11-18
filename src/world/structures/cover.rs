//! Test-only measure of how well colliders cover what is drawn: points spread over the drawn triangles, and
//! how far each lies from the nearest collider.

use avian3d::prelude::*;
use bevy::prelude::*;

/// Longest side in metres from which a drawn piece counts as a thing a body can run into: pieces drawn
/// 0.3 m long do not count, pieces of 0.34 m do.
pub(crate) const SIZE: f32 = 0.32;

/// Width in metres under which a drawn piece is a rod, a rail, a cable or a chain, too thin to block a
/// body: faces drawn 0.1 m wide are thin, faces of 0.12 m are not.
pub(crate) const THIN: f32 = 0.11;

/// Distance in metres a drawn point may lie from the nearest collider or above the ground and still count
/// as covered: less than the radius of the body of the player, so nothing fits in between.
pub(crate) const NEAR: f32 = 0.3;

/// Distance in metres between two points spread over a drawn triangle.
const STEP: f32 = 0.25;

/// Returns the triangles of the triangle list `mesh`.
pub(crate) fn triangles(mesh: &Mesh) -> Vec<[Vec3; 3]> {
    let places = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
    let points = places.and_then(|a| a.as_float3()).unwrap_or_default();
    let corners = points.as_chunks::<3>().0.iter();
    corners.map(|t| t.map(Vec3::from_array)).collect()
}

/// Returns the length and the width of `tri` as half of a drawn face: its middle side, and the height over
/// that side. Half of a box face 2 m by 0.1 m is 2 m long and 0.1 m wide, whatever its diagonal.
pub(crate) fn extent(tri: &[Vec3; 3]) -> (f32, f32) {
    let sides = [tri[1] - tri[0], tri[2] - tri[1], tri[0] - tri[2]];
    let mut lengths = sides.map(|s| s.length());
    lengths.sort_by(f32::total_cmp);
    let area = sides[0].cross(sides[1]).length();
    (lengths[1], area / lengths[1].max(1e-6))
}

/// Returns true when `tri` is large enough to be part of a thing a body can run into.
pub(crate) fn sizable(tri: &[Vec3; 3]) -> bool {
    let (long, wide) = extent(tri);
    long >= SIZE && wide >= THIN
}

/// Returns points spread over `tri`, corners included, no farther apart than [`STEP`].
pub(crate) fn spread(tri: &[Vec3; 3]) -> Vec<Vec3> {
    let sides = [tri[1] - tri[0], tri[2] - tri[0]].map(|s| s.length());
    let n = ((sides[0].max(sides[1]) / STEP).ceil() as u32).clamp(1, 40);
    let at = |i: u32, j: u32| {
        let (u, v) = (i as f32 / n as f32, j as f32 / n as f32);
        tri[0] + (tri[1] - tri[0]) * u + (tri[2] - tri[0]) * v
    };
    (0..=n)
        .flat_map(|i| (0..=n - i).map(move |j| (i, j)))
        .map(|(i, j)| at(i, j))
        .collect()
}

/// Returns the distance in metres from `point` to `solid`, which stands at the origin: zero inside it.
pub(crate) fn reach(solid: &Collider, point: Vec3) -> f32 {
    solid.distance_to_point(Vec3::ZERO, Quat::IDENTITY, point, true)
}

/// Returns the points of the sizable triangles of `tris` that lie farther than [`NEAR`] from `solid` and are
/// not `spared`, each with the index of its triangle.
pub(crate) fn bare(
    tris: &[[Vec3; 3]],
    solid: &Collider,
    spared: impl Fn(usize, Vec3) -> bool,
) -> Vec<(usize, Vec3)> {
    let large = tris.iter().enumerate().filter(|(_, tri)| sizable(tri));
    let points = large.flat_map(|(i, tri)| spread(tri).into_iter().map(move |p| (i, p)));
    points
        .filter(|(i, p)| !spared(*i, *p) && reach(solid, *p) > NEAR)
        .collect()
}
