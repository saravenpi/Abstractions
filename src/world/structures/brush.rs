//! The builder every structure draws with: a rotatable local frame, a merged mesh and a collider list.

use bevy::prelude::*;

use super::canvas::{Canvas, Hold};
use super::glow::LightPoint;
use super::site::Site;
use crate::world::Terrain;

/// Collider shape of one solid part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// Box with its full side lengths.
    Cuboid(Vec3),
    /// Upright cylinder with radius and height.
    Cylinder(f32, f32),
}

/// One solid piece in chunk space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    /// Center in chunk space.
    pub pos: Vec3,
    /// Rotation in chunk space.
    pub rot: Quat,
    /// Collider shape.
    pub shape: Shape,
}

/// A base color and whether the piece blocks the player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat {
    /// sRGB base color.
    pub rgb: [f32; 3],
    /// True when the piece also gets a collider.
    pub solid: bool,
}

impl Mat {
    /// A colored piece that blocks the player.
    pub fn solid(rgb: [f32; 3]) -> Self {
        Self { rgb, solid: true }
    }

    /// A colored piece the player walks through.
    pub fn deco(rgb: [f32; 3]) -> Self {
        Self { rgb, solid: false }
    }
}

/// Merged mesh and colliders of the structures of one site, drawn in a local frame.
///
/// Chunk space is the chart of `terrain` moved to `center`, with `y` the height above sea level. On the
/// globe `terrain` is the metric view around the site, and [`Build::finish`] carries the result to the planet.
pub struct Build<'a> {
    /// Terrain every layout and height decision reads: the metric view around the site.
    pub terrain: &'a Terrain,
    /// The chart of the chunk, whose terrain mesh is the visible ground.
    pub ground: &'a Terrain,
    /// Position of the chunk center on the chart of `terrain`.
    pub center: Vec2,
    /// Triangles in chunk space.
    pub mesh: Canvas,
    /// How the run being drawn is carried onto the planet.
    pub hold: Hold,
    /// Solid parts in chunk space.
    pub parts: Vec<Part>,
    /// Structure being drawn.
    pub site: Site,
    /// World position of the current frame.
    pub pos: Vec2,
    /// Rotation of the current frame around the vertical axis.
    pub yaw: f32,
    /// World height of local `y = 0`.
    pub base_y: f32,
    /// Quads drawn so far, used for tone variation.
    pub count: u32,
    /// Light points placed so far, in chunk space.
    pub lights: Vec<LightPoint>,
    /// Closed boxes and double-sided sheets drawn so far, as vertex ranges.
    #[cfg(test)]
    pub audit: Vec<(super::audit::Piece, usize, usize)>,
    /// Roles of the pieces drawn so far, as vertex ranges.
    #[cfg(test)]
    pub tags: Vec<(super::tags::Tag, usize, usize)>,
}

impl<'a> Build<'a> {
    /// Starts an empty build for the chunk centered at `center`, on a terrain that is both the view of the
    /// site and the chart of the chunk.
    #[cfg(test)]
    pub fn new(terrain: &'a Terrain, center: Vec2, site: Site) -> Self {
        Self::on((terrain, terrain), center, site)
    }

    /// Starts an empty build on `charts`, the chart of the chunk then the metric view around the site, for
    /// the chunk centered at `center` on that view.
    pub fn on(charts: (&'a Terrain, &'a Terrain), center: Vec2, site: Site) -> Self {
        let (ground, terrain) = charts;
        Self {
            terrain,
            ground,
            center,
            mesh: Canvas::default(),
            hold: Hold::Rigid(site.origin),
            parts: Vec::new(),
            site,
            pos: site.origin,
            yaw: site.yaw,
            base_y: site.floor,
            count: 0,
            lights: Vec::new(),
            #[cfg(test)]
            audit: Vec::new(),
            #[cfg(test)]
            tags: Vec::new(),
        }
    }

    /// Moves to the frame of another structure, which stands as one rigid piece at its own origin.
    pub fn begin(&mut self, site: Site) {
        self.seal();
        self.hold = Hold::Rigid(site.origin);
        self.site = site;
        self.pos = site.origin;
        self.yaw = site.yaw;
        self.base_y = site.floor;
    }

    /// Moves the frame by `off` (in the structure frame) and turns it by `turn` radians.
    pub fn frame(&mut self, off: Vec2, turn: f32) {
        let o = Quat::from_rotation_y(self.site.yaw) * Vec3::new(off.x, 0.0, off.y);
        self.pos = self.site.origin + Vec2::new(o.x, o.z);
        self.yaw = self.site.yaw + turn;
    }

    /// Converts a local point to chunk space.
    pub fn pt(&self, l: Vec3) -> Vec3 {
        let r = Quat::from_rotation_y(self.yaw) * Vec3::new(l.x, 0.0, l.z);
        let flat = self.pos - self.center;
        Vec3::new(flat.x + r.x, self.base_y + l.y, flat.y + r.z)
    }

    /// Returns the ground height under the local point `(x, z)`.
    pub fn ground_at(&self, x: f32, z: f32) -> f32 {
        let r = Quat::from_rotation_y(self.yaw) * Vec3::new(x, 0.0, z);
        self.terrain.height(self.pos.x + r.x, self.pos.y + r.z)
    }
}
