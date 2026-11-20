//! A watch over the herds rolled off the main thread: the old watch keeps answering while the new
//! one is built, and a roll that finishes is swapped in.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::tasks::{
    AsyncComputeTaskPool, Task, TaskPool, block_on, futures_lite::future::poll_once,
};

use super::herds::{Land, read};
use crate::life::herd::{Search, Watch};
use crate::universe::PlanetSpec;
use crate::world::Globe;

/// A watch being rolled by a task, with the seed of the globe, the unit direction and the universe
/// time it is rolled for.
pub struct Rolling {
    /// The task that rolls the watch, nothing when the globe has no life.
    task: Task<Option<Watch>>,
    /// Seed of the globe it is rolled on.
    pub seed: u64,
    /// Unit direction it is rolled around.
    pub at: DVec3,
    /// Universe time it is rolled for.
    pub time: f64,
}

/// Rolls the watch of `search` on `globe` and `planet`, on the calling thread, or returns nothing
/// when the globe has no life.
pub fn roll(globe: &Globe, planet: &PlanetSpec, search: &Search) -> Option<Watch> {
    let land = Land {
        globe,
        life: globe.life()?,
        planet,
    };
    Some(read(land, search.time, |g| {
        Watch::new(land.life, g, search)
    }))
}

impl Rolling {
    /// Starts rolling the watch of `search` on `globe` and `planet` in a task.
    pub fn start(globe: &Arc<Globe>, planet: &Arc<PlanetSpec>, search: Search) -> Self {
        let (own, planet) = (globe.clone(), planet.clone());
        let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
        Self {
            task: pool.spawn(async move { roll(&own, &planet, &search) }),
            seed: globe.seed(),
            at: search.at,
            time: search.time,
        }
    }

    /// Returns the watch with the unit direction and time it was rolled for once the task is done,
    /// waiting for it when `wait` is set, or the roll itself while it runs.
    pub fn finish(mut self, wait: bool) -> Result<Option<(Watch, DVec3, f64)>, Self> {
        let (at, time) = (self.at, self.time);
        let watch = match block_on(poll_once(&mut self.task)) {
            Some(watch) => watch,
            None if wait => block_on(self.task),
            None => return Err(self),
        };
        Ok(watch.map(|watch| (watch, at, time)))
    }
}
