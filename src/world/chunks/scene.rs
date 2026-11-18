//! What the streamer reads of the running game: where the focus is and what it sees, what time
//! it is and which entities hang under a chunk.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::reach::Reach;
use super::sight::Eye;
use crate::render::RayTracing;
use crate::world::Terrain;
use crate::world::focus::Focus;

/// The focus, the clock and the entity tree, for the systems of the streaming chain.
#[derive(SystemParam)]
pub struct Scene<'w, 's> {
    focus: Focus<'w, 's>,
    time: Option<Res<'w, Time>>,
    children: Query<'w, 's, &'static Children>,
    meshes: Query<'w, 's, &'static Mesh3d>,
    terrain: Option<Res<'w, Terrain>>,
    reach: Option<Res<'w, Reach>>,
    ray_tracing: Option<Res<'w, RayTracing>>,
}

impl Scene<'_, '_> {
    /// Returns the focus position in metres from the planet centre, in the planet frame.
    pub fn at(&self) -> Option<DVec3> {
        self.focus.planet()
    }

    /// Returns the seconds the last frame took.
    pub fn delta(&self) -> f64 {
        self.time.as_ref().map_or(0.0, |time| time.delta_secs_f64())
    }

    /// Returns the seconds the game has run.
    pub fn now(&self) -> f64 {
        self.time
            .as_ref()
            .map_or(0.0, |time| time.elapsed_secs_f64())
    }

    /// Returns what the focus can see through the fog, or nothing without a focus on a planet.
    /// With ray tracing on it sees nothing, so nothing fades: the traced image ignores dither.
    pub fn eye(&self) -> Option<Eye> {
        let (at, terrain) = (self.at()?, self.terrain.as_ref()?);
        let reach = self.reach.as_deref().copied().unwrap_or(Reach::of(None));
        let traced = self.ray_tracing.as_ref().is_some_and(|on| on.0);
        let ground = f64::from(reach.sight(terrain.globe()));
        Some(Eye::of(terrain.globe(), at, ground, !traced))
    }

    /// Returns the mesh `entity` draws.
    pub fn mesh(&self, entity: Entity) -> Option<Handle<Mesh>> {
        self.meshes.get(entity).ok().map(|mesh| mesh.0.clone())
    }

    /// Returns `root` and every entity under it: all that may draw a mesh of a chunk.
    pub fn parts(&self, root: Entity) -> Vec<Entity> {
        let below = self.children.iter_descendants(root);
        std::iter::once(root).chain(below).collect()
    }
}
