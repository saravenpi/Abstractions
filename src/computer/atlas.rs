//! The atlas: what the player has visited, and what that lets them know.

use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::reveal::{galaxies_around, home_galaxies, systems_around};
use super::route::Target;
use crate::universe::{BodyId, GalaxyId, GalaxySpec, StarId, StarSpec, Universe};

/// What the player knows of the universe.
///
/// Only the visited ids are stored. Everything a visit reveals is derived again from those ids
/// and the universe seed, so the record grows by one id per visit, whatever it reveals.
#[derive(Resource, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Atlas {
    /// Galaxies the player has been inside, oldest first.
    pub galaxies: Vec<GalaxyId>,
    /// Systems the player has been inside, oldest first.
    pub systems: Vec<StarId>,
    /// Planets the player has flown down to, oldest first.
    pub bodies: Vec<BodyId>,
    /// Where the computer is set to go, copied from [`super::Destination`].
    pub destination: Option<Target>,
    /// Planets the player has walked on, oldest first. The companion celebrates each new one
    /// once, in any session. A file from before this list has none.
    pub walked: Vec<BodyId>,
}

impl Atlas {
    /// Returns the atlas of a new game: the home planet, system and galaxy are visited.
    pub fn new(universe: &Universe) -> Self {
        let home = universe.home();
        Self {
            galaxies: vec![home.star.galaxy],
            systems: vec![home.star],
            bodies: vec![home],
            destination: None,
            walked: vec![home],
        }
    }

    /// Records that the player walks on `body`. Returns true the first time on a planet that
    /// is not the first of the list: a world whose list is empty takes the planet silently.
    pub fn walk(&mut self, body: BodyId) -> bool {
        if self.walked.contains(&body) {
            return false;
        }
        self.walked.push(body);
        self.walked.len() > 1
    }

    /// Returns true when the player has been at `target`.
    pub fn visited(&self, target: Target) -> bool {
        match target {
            Target::Body(id) => self.bodies.contains(&id),
            Target::Star(id) => self.systems.contains(&id),
            Target::Galaxy(id) => self.galaxies.contains(&id),
        }
    }

    /// Records a visit to `target`. Returns true when it is the first one. Nothing is ever removed.
    pub fn visit(&mut self, target: Target) -> bool {
        if self.visited(target) {
            return false;
        }
        match target {
            Target::Body(id) => self.bodies.push(id),
            Target::Star(id) => self.systems.push(id),
            Target::Galaxy(id) => self.galaxies.push(id),
        }
        true
    }

    /// Returns every galaxy the player knows, the home galaxy first.
    ///
    /// The home galaxy gives its nearest neighbours. Every other visited galaxy reveals the
    /// galaxies within [`super::REVEAL_CELLS`] cells of it.
    pub fn known_galaxies(&self, universe: &Universe) -> Vec<GalaxySpec> {
        let home = universe.home().star.galaxy;
        let mut known = home_galaxies(universe);
        for &id in self.galaxies.iter().filter(|&&id| id != home) {
            known.extend(galaxies_around(universe, id));
        }
        let mut seen = HashSet::new();
        known.retain(|galaxy| seen.insert(galaxy.id));
        known
    }

    /// Returns every system of `galaxy` the player knows: the visited ones, those within
    /// [`super::REVEAL_GLY`] game light-years of a visited one, and always the home star.
    pub fn known_systems(&self, universe: &Universe, galaxy: GalaxyId) -> Vec<StarSpec> {
        let mut known = Vec::new();
        for &id in self.systems.iter().filter(|id| id.galaxy == galaxy) {
            known.extend(systems_around(universe, id));
        }
        let home = universe.home().star;
        known.extend(universe.star(home).filter(|_| home.galaxy == galaxy));
        let mut seen = HashSet::new();
        known.retain(|star| seen.insert(star.id));
        known
    }
}
