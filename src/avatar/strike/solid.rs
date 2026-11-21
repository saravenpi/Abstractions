//! Solids: what a tool meets in the world. The colliders of rocks, trees, the base, the ship and the
//! ground near the robot, and the drawn ground where no collider is loaded. A collider on the drawn
//! ground is that ground and takes its soil; any other one is hard.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::javelin::{Ground, Planet, Soil, cross};
use crate::space::Anchor;
use crate::world::{Chunks, NOT_ANIMALS, Terrain};

/// Metres from the drawn ground within which a collider counts as the ground itself.
const ON_GROUND: f32 = 0.5;

/// What a tool reads of the world besides its colliders: where render space sits, the height field
/// and the meshes drawn.
pub type Land<'w> = (
    Option<Res<'w, Anchor>>,
    Option<Res<'w, Terrain>>,
    Option<Res<'w, Chunks>>,
);

/// Where a segment first meets the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Touch {
    /// How far along the segment, 0 at its start to 1 at its end.
    pub share: f32,
    /// The point, in render space.
    pub at: Vec3,
    /// The unit normal there, in render axes.
    pub normal: Vec3,
    /// What it is.
    pub soil: Soil,
}

/// Something a segment can meet.
pub trait Meets {
    /// Returns where the segment from `from` to `to` first meets it, if it does.
    fn meet(&self, from: Vec3, to: Vec3) -> Option<Touch>;
}

/// The world a tool moves through: the colliders but those of its own body and of the animals, whose
/// parts a blow reaches by its own aim, and the drawn ground.
pub struct Solids<'a, 'w, 's> {
    /// The colliders.
    world: &'a SpatialQuery<'w, 's>,
    /// The drawn ground, when the world has one.
    ground: Option<Planet<'a>>,
    /// The colliders left out: the body that holds or threw the tool.
    filter: SpatialQueryFilter,
}

impl<'a, 'w, 's> Solids<'a, 'w, 's> {
    /// Returns the world of `world` and `land` for a tool of the body `own`.
    pub fn new(world: &'a SpatialQuery<'w, 's>, land: &'a Land, own: Entity) -> Self {
        let (anchor, terrain, chunks) = land;
        let ground = anchor
            .as_deref()
            .zip(terrain.as_deref())
            .map(|(anchor, terrain)| Planet::new(anchor, terrain, chunks.as_deref()));
        let filter = SpatialQueryFilter::from_mask(NOT_ANIMALS).with_excluded_entities([own]);
        Self {
            world,
            ground,
            filter,
        }
    }

    /// Returns where the segment from `from` to `to` meets a collider, if it does.
    fn collider(&self, from: Vec3, to: Vec3) -> Option<Touch> {
        let (way, length) = (Dir3::new(to - from).ok()?, from.distance(to));
        let hit = self.world.cast_ray(from, way, length, true, &self.filter)?;
        let at = from + way * hit.distance;
        let near = |g: &Planet| g.clearance(at).abs() < ON_GROUND;
        let soil = match self.ground.as_ref().filter(|g| near(g)) {
            Some(ground) => ground.soil(at),
            None => Soil::Hard,
        };
        let share = hit.distance / length.max(f32::EPSILON);
        Some(Touch {
            share,
            at,
            normal: hit.normal,
            soil,
        })
    }
}

impl Meets for Solids<'_, '_, '_> {
    /// Returns the nearer of the collider and the drawn ground the segment meets.
    fn meet(&self, from: Vec3, to: Vec3) -> Option<Touch> {
        let drawn = self.ground.as_ref().and_then(|ground| {
            let at = cross(ground, from, to)?;
            let share = from.distance(at) / from.distance(to).max(f32::EPSILON);
            let (normal, soil) = (ground.normal(at), ground.soil(at));
            Some(Touch {
                share,
                at,
                normal,
                soil,
            })
        });
        match (self.collider(from, to), drawn) {
            (Some(a), Some(b)) if b.share < a.share => Some(b),
            (Some(a), _) => Some(a),
            (None, b) => b,
        }
    }
}
