//! Chunk streaming: a quadtree of coarse terrain nodes over the whole planet, and detail chunks
//! with props, structures and colliders built off-thread around the focus.

mod budget;
mod build;
mod calm;
mod cut;
mod detail;
pub mod dither;
mod embed;
mod face;
mod fade;
mod far;
mod grow;
mod lane;
mod motion;
mod near;
mod nodes;
mod pace;
mod place;
mod plan;
mod reach;
mod scene;
mod sight;
mod spawn;
mod stream;
mod stretch;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bodies;
#[cfg(test)]
mod tests_embed;
#[cfg(test)]
mod tests_fade;
#[cfg(test)]
mod tests_fly;
#[cfg(test)]
mod tests_lane;
#[cfg(test)]
mod tests_reach;
#[cfg(test)]
mod tests_stream;
mod turf;
mod work;

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::tasks::Task;

pub use self::budget::{Budget, follow_setting, set_budget};
use self::build::ChunkData;
pub use self::calm::{Calm, watch_calm};
pub use self::cut::show;
pub use self::detail::refine;
pub use self::face::{follow_face, rehome};
use self::fade::Fades;
use self::far::FarView;
pub use self::far::watch_far;
pub use self::grow::grow;
use self::lane::Lane;
use self::motion::Motion;
pub use self::near::leaf_arc;
use self::near::{Around, Course};
use self::nodes::Nodes;
pub use self::place::{Sited, place};
use self::plan::Loaded;
pub use self::reach::{Reach, STEPS};
use self::spawn::Spawner;
pub use self::stream::{collect, mark_ready, stream};
use self::stretch::Stretch;
use super::globe::quadtree::Tree;
use super::globe::{ChunkId, Face};
use super::ground::GroundMaterial;
use super::props::PropAssets;

/// Finished chunks turned into entities per frame once the world is ready, at the default render
/// distance and the time budget allowing.
const FRAME_BUDGET: usize = 2;

/// What the detail chunks were last settled for: nothing to do until one of these changes.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Settled {
    face: Option<Face>,
    lane: Lane,
    rings: i32,
    revision: u64,
}

/// The quadtree, its coarse nodes, and the detail chunks that exist or are still being generated.
#[derive(Resource, Default)]
pub struct Chunks {
    tree: Tree,
    revision: u64,
    nodes: Nodes,
    far: FarView,
    loaded: HashMap<ChunkId, Loaded>,
    pending: HashMap<ChunkId, Task<ChunkData>>,
    settled: Option<Settled>,
    stamp: u64,
    grown: Option<((IVec3, IVec3), u64, i32)>,
    drawn: Option<u64>,
    motion: Motion,
    fades: Fades,
    stretch: Stretch,
}

impl Chunks {
    /// Forgets every chunk and cancels running tasks.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Despawns the quadtree nodes and the detail chunks and cancels their tasks.
    fn drop_near(&mut self, commands: &mut Commands) {
        for loaded in self.loaded.values() {
            commands.entity(loaded.entity).try_despawn();
        }
        self.nodes.clear(commands);
        self.fades.clear(commands);
        let far = std::mem::take(&mut self.far);
        *self = Self {
            far,
            revision: self.revision + 1,
            ..default()
        };
    }

    /// Despawns every chunk and cancels running tasks, so the streamer builds them all again.
    pub fn rebuild(&mut self, commands: &mut Commands) {
        self.drop_near(commands);
        self.far.clear(commands);
    }

    /// Notes that a node or a chunk appeared or went: the draw cut and the quadtree look again.
    fn touch(&mut self) {
        self.stamp += 1;
    }

    /// Returns the chunk coordinates around a focus at planet position `at`, with the lane to
    /// the point it will reach at its speed and the detail its pace leaves time for. A planet
    /// without ground has no detail chunk, so nothing around any focus.
    fn around(&self, spawner: &Spawner, at: DVec3) -> Option<Around> {
        if !spawner.terrain.globe().has_ground() {
            return None;
        }
        let course = Course {
            ahead: self.motion.ahead(at, spawner.terrain.globe().radius()),
            rings: spawner.reach().rings,
            pace: self.motion.pace,
        };
        Around::along(&spawner.terrain, at, &course)
    }

    /// Returns the chunks whose generation task is done.
    fn finished(&self) -> Vec<ChunkId> {
        let done = self.pending.iter().filter(|(_, task)| task.is_finished());
        done.map(|(c, _)| *c).collect()
    }
}

/// Creates the shared material.
pub fn setup_assets(
    mut commands: Commands,
    mut mats: ResMut<Assets<GroundMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(PropAssets::new(&mut mats, &mut images));
}

/// Drops every chunk of the planet that was streamed before the planet changed.
pub fn change_planet(mut commands: Commands, mut chunks: ResMut<Chunks>) {
    chunks.rebuild(&mut commands);
}
