//! The far planet: one baked mesh shown while the quadtree is not yet covering the globe.

use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::Chunks;
use super::build::{NodeData, render_only};
use super::dither::Way;
use super::horizon::Bounds;
use super::nodes::Node;
use super::scene::Scene;
use super::spawn::Spawner;
use super::stretch::Stretch;
use crate::world::globe::far::{Range, bake, seen};
use crate::world::globe::{ChunkId, Face};
use crate::world::mesh::Frame;

/// The baked mesh of the whole planet and whether it stands in for the quadtree.
#[derive(Default)]
pub struct FarView {
    node: Option<Node>,
    task: Option<Task<(Mesh, Option<Aabb>)>>,
    active: bool,
}

impl FarView {
    /// Returns true while the baked planet replaces the quadtree.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Despawns the baked planet and cancels its task.
    pub fn clear(&mut self, commands: &mut Commands) {
        if let Some(node) = &self.node {
            commands.entity(node.entity).try_despawn();
        }
        *self = Self::default();
    }

    /// Bakes the planet off-thread once, and spawns it hidden when the mesh is done.
    fn prepare(&mut self, commands: &mut Commands, spawner: &mut Spawner) {
        if self.node.is_some() {
            return;
        }
        let globe = spawner.terrain.globe().clone();
        let pool = AsyncComputeTaskPool::get();
        let task = self.task.get_or_insert_with(|| {
            pool.spawn(async move {
                let mesh = bake(&globe);
                let aabb = mesh.get_aabb();
                (render_only(mesh), aabb)
            })
        });
        if let Some((mesh, aabb)) = block_on(poll_once(task)) {
            let frame = Frame::PLANET;
            let bounds = Bounds::WHOLE;
            self.node = Some(spawner.coarse(
                commands,
                NodeData {
                    mesh,
                    frame,
                    bounds,
                    aabb,
                },
            ));
            self.task = None;
        }
    }

    /// Shows or hides the baked planet.
    fn set_visible(&mut self, commands: &mut Commands, visible: bool) {
        if let Some(node) = self.node.as_mut().filter(|node| node.visible != visible) {
            node.visible = visible;
            node.show(commands);
        }
    }
}

/// Returns true when the six whole faces have a mesh, so the quadtree can draw the full globe.
fn covered(chunks: &Chunks) -> bool {
    Face::ALL.iter().all(|face| {
        chunks.nodes.has(ChunkId {
            face: *face,
            level: 0,
            x: 0,
            y: 0,
        })
    })
}

impl Chunks {
    /// Returns true when the world already draws an opaque surface over the whole planet,
    /// either through the baked far mesh or through all six root quadtree faces.
    pub fn covers_globe(&self) -> bool {
        self.far.node.as_ref().is_some_and(|node| node.visible)
            || (self.drawn.is_some() && covered(self))
    }
}

/// Drops the quadtree and the detail chunks at the hand-over, and lets the nodes on the screen
/// dither out over a fade instead of vanishing on one frame: the sky draws the ball of the planet
/// under them, sunk just under the sea, so the ground of the world melts into it. Hidden nodes go
/// at once. `now` is the time of the fades in seconds.
fn hand_over(chunks: &mut Chunks, commands: &mut Commands, now: f64) {
    let live = std::mem::take(&mut chunks.nodes.live);
    let span = chunks.fades.span;
    chunks.drop_near(commands);
    chunks.fades.span = span;
    for node in live.into_values() {
        match node.visible {
            true => {
                let at = (Way::Leave, now);
                chunks
                    .fades
                    .begin(commands, node.entity, vec![node.entity], at);
            }
            false => commands.entity(node.entity).try_despawn(),
        }
    }
}

/// Drops the quadtree beyond the hand-over of a radius and a half, where the sky draws the planet,
/// and streams it again under a radius and two fifths. The baked planet only stands in while the
/// quadtree has not covered the globe yet: in between the sky draws its sharper ball.
///
/// As the first step of the chain that sees the focus, it also notes how the focus moves and
/// how far the budget of the frame stretches for the work that waits.
pub fn watch_far(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    scene: Scene,
    mut spawner: Spawner,
) {
    let chunks = &mut *chunks;
    let (at, waiting) = (
        scene.at(),
        chunks.pending.len() + chunks.nodes.building.len(),
    );
    chunks.motion.advance(at, scene.delta());
    chunks.stretch = Stretch::of(waiting, scene.delta());
    chunks.far.prepare(&mut commands, &mut spawner);
    let globe = spawner.terrain.globe();
    let seen = at.map_or(Range::Beyond, |at| seen(globe, at, chunks.far.active));
    let far = seen != Range::Near;
    if far && !chunks.far.active {
        hand_over(chunks, &mut commands, scene.now());
    }
    chunks.far.active = far;
    let visible = seen == Range::Near && !covered(chunks);
    chunks.far.set_visible(&mut commands, visible);
}
