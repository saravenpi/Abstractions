//! Built floors and nearby sites: what a structure gives the player to stand on, and what stands around a
//! point, as plain data in the frame of the planet.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::site::{Kind, Site};
use crate::world::Terrain;
use crate::world::mesh::Frame;

/// What a built floor is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorKind {
    /// Planks: house floors, stilt decks, piers and plank walkways.
    Wood,
    /// Masonry: foundation platforms, paved plazas and stone steps.
    Stone,
    /// Sandstone and adobe.
    Sand,
    /// Packed earth of streets and footpaths.
    Earth,
}

/// A structure or a village near a position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nearby {
    /// What stands there.
    pub kind: Kind,
    /// Its origin at floor height, in meters from the planet center, in the frame of the planet.
    pub pos: DVec3,
    /// How far its pieces reach from the origin, in meters.
    pub reach: f32,
    /// Distance in meters from the asked position to the origin.
    pub distance: f32,
}

/// One flat built floor in a site frame: round or rectangular, the two corners or the radius in `hi.x`,
/// the lowest and highest standing height above the site floor, and the material.
pub type Slab = (bool, [f32; 2], [f32; 2], [f32; 2], FloorKind);

/// A flat built floor: a rectangle or a disc in a rigid frame fixed to the planet.
#[derive(Debug, Clone, Copy)]
pub struct Pad {
    frame: Frame,
    lo: Vec2,
    hi: Vec2,
    y: (f32, f32),
    round: bool,
    kind: FloorKind,
    /// True for the floor of a room, false for a floor under the open sky.
    room: bool,
}

impl Pad {
    /// Returns the floor `slab` of a piece that stands at the chart point `at` of `view`, turned by `yaw`, with
    /// its floor `floor` meters above sea level. It is the floor of a room until it is made [`Pad::open`].
    pub fn new(view: &Terrain, at: (Vec2, f32, f32), slab: &Slab) -> Self {
        let (origin, yaw, floor) = at;
        let frame = Frame::at(view, origin);
        Self {
            frame: Frame {
                origin: frame.origin,
                rot: frame.rot * DQuat::from_rotation_y(f64::from(yaw)),
            },
            lo: Vec2::from(slab.1),
            hi: Vec2::from(slab.2),
            y: (floor + slab.3[0] - BELOW, floor + slab.3[1] + ABOVE),
            round: slab.0,
            kind: slab.4,
            room: true,
        }
    }

    /// Returns this floor as one under the open sky: a platform, a plaza or a boardwalk.
    pub fn open(self) -> Self {
        Self {
            room: false,
            ..self
        }
    }

    /// Returns the material when feet at the planet position `pos` stand on this floor and it is the floor
    /// of a room.
    pub fn room(&self, pos: DVec3) -> Option<FloorKind> {
        self.under(pos).filter(|_| self.room)
    }

    /// Returns the material when feet at the planet position `pos` stand on this floor.
    pub fn under(&self, pos: DVec3) -> Option<FloorKind> {
        let local = self.frame.local(pos);
        let flat = local.xz();
        let inside = match self.round {
            true => flat.length() <= self.hi.x,
            false => flat.cmpge(self.lo).all() && flat.cmple(self.hi).all(),
        };
        (inside && (self.y.0..=self.y.1).contains(&local.y)).then_some(self.kind)
    }
}

/// How far below a floor feet may be and still stand on it, in meters.
const BELOW: f32 = 0.3;

/// How far above a floor feet may be and still stand on it, in meters.
const ABOVE: f32 = 0.6;

/// A strip of built ground that follows the terrain: a street or a footpath between two points.
#[derive(Debug, Clone, Copy)]
pub struct Strip {
    /// Planet position of one end, on the ground.
    pub from: DVec3,
    /// Planet position of the other end, on the ground.
    pub to: DVec3,
    /// Direction away from the planet center at the strip.
    pub up: DVec3,
    /// Half width in meters.
    pub half: f32,
    /// Material.
    pub kind: FloorKind,
}

impl Strip {
    /// Returns the material when feet at the planet position `pos` stand on this strip.
    pub fn under(&self, pos: DVec3) -> Option<FloorKind> {
        let along = self.to - self.from;
        let k = ((pos - self.from).dot(along) / along.length_squared().max(1e-9)).clamp(0.0, 1.0);
        let off = pos - (self.from + along * k);
        let rise = off.dot(self.up);
        let near = (off - self.up * rise).length() <= f64::from(self.half);
        (near && rise.abs() <= 1.5).then_some(self.kind)
    }
}

/// Returns the built floors of a small structure of `kind`, in its site frame.
pub fn slabs(kind: Kind) -> &'static [Slab] {
    use FloorKind::{Sand, Stone, Wood};
    match kind {
        Kind::StoneCircle => &[(true, [0.0; 2], [2.0, 0.0], [0.25; 2], Stone)],
        Kind::Tower => &[(true, [0.0; 2], [5.1, 0.0], [0.0; 2], Stone)],
        Kind::Cabin => &[(false, [-3.3, -2.7], [3.3, 2.7], [0.0; 2], Stone)],
        Kind::Lighthouse => &[(true, [0.0; 2], [3.6, 0.0], [0.0; 2], Stone)],
        Kind::Pyramid => &[(false, [-9.6; 2], [9.6; 2], [0.0, 3.4], Sand)],
        Kind::Obelisk => &[(false, [-2.2; 2], [2.2; 2], [0.3, 0.6], Stone)],
        Kind::Dock => &[
            (false, [-1.1, 0.0], [1.1, 14.0], [0.0; 2], Wood),
            (false, [-2.4, 14.0], [2.4, 16.0], [0.0; 2], Wood),
        ],
        Kind::StiltHut => &[(false, [-2.6, -2.2], [2.6, 2.2], [0.0; 2], Wood)],
        Kind::Shrine => &[(false, [-3.0; 2], [3.0; 2], [0.0; 2], Stone)],
        Kind::Camp | Kind::Village => &[],
    }
}

/// Returns the pads of the small structure at `site`, laid out on `view`, the metric view around it.
pub fn site_pads<'a>(view: &'a Terrain, site: &Site) -> impl Iterator<Item = Pad> + 'a {
    let at = (site.origin, site.yaw, site.floor);
    slabs(site.kind)
        .iter()
        .map(move |slab| Pad::new(view, at, slab))
}
