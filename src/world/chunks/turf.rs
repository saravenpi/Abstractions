//! Ground cover on demand: the grass and flowers of a chunk exist only while the focus is near.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::tasks::{Task, block_on, futures_lite::future::poll_once};

use super::budget::Spend;
use super::spawn::Spawner;

/// Rings past the last ring that shows ground cover at which a chunk drops its cover mesh. It is
/// built again two rings closer, so a focus that paces a border builds nothing.
const DROP: i32 = 3;

/// Where the ground cover of a chunk stands.
enum State {
    /// Not built, or dropped when the focus went far.
    Missing,
    /// Being built off-thread.
    Building(Task<Option<Mesh>>),
    /// Built: the child entity that draws it, or nothing for a chunk without ground cover.
    Ready(Option<Entity>),
}

/// The ground cover of one loaded chunk.
pub struct Turf {
    state: State,
    shown: bool,
}

impl Turf {
    /// Returns the cover of a new chunk: `built` tells whether its generation included the
    /// ground cover, whose child entity is `grass`.
    pub fn new(grass: Option<Entity>, built: bool) -> Self {
        let state = match built {
            true => State::Ready(grass),
            false => State::Missing,
        };
        Self { state, shown: true }
    }

    /// Returns true when a chunk at `rank` should build its cover now: it is missing and the focus
    /// is within one ring of the last ring `cover` that shows it.
    pub fn wanted(&self, rank: i32, cover: i32) -> bool {
        matches!(self.state, State::Missing) && rank <= cover + 1
    }

    /// Returns the cover entity once the cover is built, or nothing inside when nothing grows.
    #[cfg(test)]
    pub fn grown(&self) -> Option<Option<Entity>> {
        match self.state {
            State::Ready(grass) => Some(grass),
            _ => None,
        }
    }

    /// Returns true when the cover has a mesh and shows it.
    pub fn drawn(&self) -> bool {
        matches!(self.state, State::Ready(Some(_))) && self.shown
    }

    /// Returns true while the cover is being built.
    pub fn building(&self) -> bool {
        matches!(self.state, State::Building(_))
    }

    /// Remembers the task that builds the cover.
    pub fn start(&mut self, task: Task<Option<Mesh>>) {
        self.state = State::Building(task);
    }

    /// Spawns a finished cover mesh as a child of `chunk`, the budget allowing.
    fn adopt(&mut self, commands: &mut Commands, chunk: Entity, spawner: &mut Spawner) {
        let State::Building(task) = &mut self.state else {
            return;
        };
        let Some(mesh) = block_on(poll_once(task)) else {
            return;
        };
        let material = MeshMaterial3d(spawner.assets.material.clone());
        let part = |mesh: Mesh| (Mesh3d(spawner.meshes.add(mesh)), material, NotShadowCaster);
        let child = mesh.map(|mesh| commands.spawn((part(mesh), ChildOf(chunk))).id());
        (self.state, self.shown) = (State::Ready(child), true);
    }

    /// Runs the cover of `chunk` at `rank` rings from the line of flight: a finished mesh is
    /// spawned while `spend` allows, the cover shows within `cover` rings and is dropped far
    /// beyond. Returns the cover entity when it came into view just now.
    pub fn tend(
        &mut self,
        commands: &mut Commands,
        chunk: Entity,
        rings: (i32, i32),
        (spawner, spend): (&mut Spawner, &mut Spend),
    ) -> Option<Entity> {
        let ((rank, cover), before) = (rings, self.drawn());
        if matches!(&self.state, State::Building(task) if task.is_finished()) && spend.more() {
            self.adopt(commands, chunk, spawner);
        }
        let State::Ready(Some(grass)) = self.state else {
            return None;
        };
        if rank > cover + DROP {
            commands.entity(grass).try_despawn();
            self.state = State::Missing;
        } else if (rank <= cover) != self.shown {
            self.shown = rank <= cover;
            let visibility = [Visibility::Hidden, Visibility::Inherited][usize::from(self.shown)];
            commands.entity(grass).try_insert(visibility);
        }
        (self.drawn() && !before).then_some(grass)
    }
}
