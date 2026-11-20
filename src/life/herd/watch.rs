//! A watch over the herds and hunts around one place: rolled once, then asked for any time of the next
//! [`AHEAD`] seconds.
//!
//! Rolling the herds and planning their hunts costs about a millisecond. Asking a watch for a herd at
//! a time costs a few microseconds, since its waypoints are drawn once and kept, so the runtime can
//! keep one watch per place and ask it every frame.

use super::{Ground, Herd, Scene, Search, Store};
use crate::life::Life;
use crate::life::hunt::{self, Carcass, Hunt};

/// Seconds after the search time a watch answers for.
pub const AHEAD: f64 = 120.0;

/// The herds and hunts that can matter within the radius of one search.
#[derive(Debug, Clone)]
pub struct Watch {
    /// The search it was rolled for.
    search: Search,
    /// Its herds and the waypoints drawn for them so far.
    store: Store,
    /// Every hunt that can matter from [`hunt::MEMORY`] seconds before the search time to [`AHEAD`]
    /// seconds after it, by start time.
    hunts: Vec<Hunt>,
}

impl Watch {
    /// Rolls the herds of `life` on `ground` that can matter within the radius of `search`, and plans
    /// their hunts.
    pub fn new(life: &Life, ground: &Ground, search: &Search) -> Self {
        let mut store = Store::roll(life, ground, search);
        let span = (search.time - hunt::MEMORY, search.time + AHEAD);
        let scene = Scene::open(life, ground, &mut store);
        let hunts = hunt::plan(&scene, span, (search.at, search.radius));
        Self {
            search: *search,
            store,
            hunts,
        }
    }

    /// Returns true when the watch answers for universe `time`: from the search time to [`AHEAD`]
    /// seconds after it.
    pub fn covers(&self, time: f64) -> bool {
        (self.search.time..=self.search.time + AHEAD).contains(&time)
    }

    /// Returns every herd with a living member whose centre is within the search radius at `time`, in
    /// a stable order.
    pub fn herds(&mut self, life: &Life, ground: &Ground, time: f64) -> Vec<Herd> {
        let (at, radius) = (self.search.at, self.search.radius);
        let scene = Scene::open(life, ground, &mut self.store);
        scene
            .roams
            .iter()
            .filter(|r| scene.metres(r.home, at) <= radius + r.reach + scene.side)
            .filter_map(|r| super::herd(&scene, r, &self.hunts, time))
            .filter(|h| h.count > 0 && scene.metres(h.centre, at) <= radius)
            .collect()
    }

    /// Returns herd `id` at `time`, wherever it has gone, or nothing when the watch does not hold it.
    pub fn herd(&mut self, life: &Life, ground: &Ground, id: u64, time: f64) -> Option<Herd> {
        let scene = Scene::open(life, ground, &mut self.store);
        let roam = scene.roams.iter().find(|r| r.id == id)?;
        super::herd(&scene, roam, &self.hunts, time)
    }

    /// Returns the hunts under way, or whose carcass still has meat, at `time`, with the hunters or the
    /// kill within the search radius, in a stable order.
    pub fn hunts(&self, life: &Life, time: f64) -> Vec<Hunt> {
        hunt::seen(
            life,
            &self.hunts,
            &Search {
                time,
                ..self.search
            },
        )
    }

    /// Returns the carcasses and bones within the search radius at `time`, in a stable order.
    pub fn carcasses(&self, life: &Life, time: f64) -> Vec<Carcass> {
        hunt::remains(
            life,
            &self.hunts,
            &Search {
                time,
                ..self.search
            },
        )
    }
}
