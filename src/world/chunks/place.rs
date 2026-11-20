//! Placing planet-fixed entities: their pose on the planet, turned into the anchor frame and the
//! render transform.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::Chunks;
use crate::space::{Anchor, Frame as Space, Placed};
use crate::world::focus::{Frames, Link};
use crate::world::globe::quadtree::leaf_at;
use crate::world::mesh::Frame;
use crate::world::{Globe, Planet};

/// Pose of an entity fixed to the streamed planet, in the planet frame.
///
/// Chunks carry it next to [`Placed`], which holds the same pose in the frame of the anchor.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sited {
    /// Position in metres from the planet centre.
    pub pos: DVec3,
    /// Rotation from the axes of the entity to planet axes.
    pub rot: DQuat,
}

impl From<Frame> for Sited {
    /// Returns the pose of an entity whose local coordinates are those of `frame`.
    fn from(frame: Frame) -> Self {
        Self {
            pos: frame.origin,
            rot: frame.rot,
        }
    }
}

/// Returns the pose of `sited` in the anchor frame and its render transform.
pub fn pose(anchor: &Anchor, link: &Link, sited: &Sited) -> (Placed, Transform) {
    let placed = Placed {
        pos: link.to_anchor(sited.pos),
        rot: link.turn * sited.rot,
    };
    let transform = Transform {
        translation: anchor.to_render(placed.pos),
        rotation: anchor.rotation_to_render(placed.rot),
        scale: Vec3::ONE,
    };
    (placed, transform)
}

/// Run condition of [`place`]: the anchor was rebased or replaced, the planet changed, or the anchor is in
/// the frame of the star, where the planet turns and orbits and every pose moves each frame.
pub fn due(anchor: Option<Res<Anchor>>, planet: Option<Res<Planet>>) -> bool {
    anchor.is_some_and(|a| a.is_changed() || matches!(a.frame, Space::System(_)))
        || planet.is_some_and(|p| p.is_changed())
}

/// Keeps the anchor frame pose and the transform of every planet-fixed entity up to date: after
/// a rebase on the surface, and every frame while the anchor is in the frame of the star.
pub fn place(frames: Frames, mut sited: Query<(&Sited, &mut Placed, &mut Transform)>) {
    let Some((anchor, link)) = frames.link() else {
        return;
    };
    for (site, mut placed, mut transform) in &mut sited {
        let (pose, wanted) = pose(&anchor, &link, site);
        if *placed != pose || *transform != wanted {
            *placed = pose;
            *transform = wanted;
        }
    }
}

impl Chunks {
    /// Returns true when the detail chunk under unit direction `dir` of `globe` has its colliders on, which
    /// a body released there stands on.
    pub fn solid_under(&self, globe: &Globe, dir: DVec3) -> bool {
        let leaf = leaf_at(dir, globe.leaf_level());
        self.loaded.get(&leaf).is_some_and(|loaded| loaded.solid)
    }
}
