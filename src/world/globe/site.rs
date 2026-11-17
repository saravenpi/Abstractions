//! Sites: positions on the ground, on the legacy plane or on the sphere, with their local metric moves.

mod ball;
mod plane;

use bevy::math::{IVec3, Vec2};

use crate::world::islands::Point;

/// A position on the ground of a planet.
///
/// `Vec2` is a point `(x, z)` of the legacy plane. `DVec3` is a point of the sea level sphere, in
/// metres from the planet centre. Ground rules that need distances are written once over this trait.
pub trait Site: Copy {
    /// The point at which the noise fields are read for this site.
    type Sample: Point;

    /// Returns the point at which the noise fields are read.
    fn point(self) -> Self::Sample;

    /// Returns the grid cells of `size` metres that may hold something within `size` of this site.
    fn cells(self, size: f32) -> impl Iterator<Item = IVec3>;

    /// Returns the site of `cell` in a grid of `size` metres, on the same ground as this site, or
    /// nothing when the cell holds no ground. `roll(slot)` gives the cell's number for a slot,
    /// each a share of the cell width.
    ///
    /// Cells hold one site per `size` squared metres of ground on average.
    fn anchor(self, cell: IVec3, size: f32, roll: impl Fn(u32) -> f32) -> Option<Self>;

    /// Returns where this site lies in the local metric frame of `center`, relative to it.
    fn offset(self, center: Self) -> Vec2;

    /// Returns the site `by` metres away along the two axes of the local frame of this site.
    fn moved(self, by: Vec2) -> Self;
}
