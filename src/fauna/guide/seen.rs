//! Which species the player has seen. An animal counts as seen once a drawn [`Creature`] of it
//! came within [`SIGHT`] metres of the robot; the plants of a living planet count as seen once
//! the robot stands on it. The list lasts for the session: saves have no field for it yet.

use std::collections::HashSet;

use bevy::prelude::*;

use super::super::Creature;
use crate::life::{Guild, Life, SpeciesId};
use crate::player::Player;
use crate::world::Terrain;

/// Distance in metres within which a creature counts as seen.
pub const SIGHT: f32 = 40.0;

/// The species the player has seen this session, on every planet.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Seen(HashSet<SpeciesId>);

impl Seen {
    /// Marks `species` as seen and returns true when it was not seen before. The live runtime
    /// may call it for any sighting the guide does not catch itself.
    pub fn mark(&mut self, species: SpeciesId) -> bool {
        self.0.insert(species)
    }

    /// Returns true when `species` was seen.
    pub fn has(&self, species: SpeciesId) -> bool {
        self.0.contains(&species)
    }

    /// Returns how many species of `life` were seen.
    pub fn count(&self, life: &Life) -> usize {
        life.species.iter().filter(|s| self.has(s.id)).count()
    }
}

/// Returns the species of `life` the robot sees from where it stands: every plant, and the
/// species of each creature at `creatures` within [`SIGHT`] of `robot`.
pub fn sighted<'a>(
    life: &'a Life,
    robot: Vec3,
    creatures: impl Iterator<Item = (SpeciesId, Vec3)> + 'a,
) -> impl Iterator<Item = SpeciesId> + 'a {
    let plants = life
        .species
        .iter()
        .filter(|s| s.role.guild == Guild::Producer)
        .map(|s| s.id);
    let near = creatures
        .filter(move |(_, at)| at.distance_squared(robot) <= SIGHT * SIGHT)
        .map(|(species, _)| species);
    plants.chain(near)
}

/// Marks what the robot on foot sees on its planet. It changes [`Seen`] only for a new species.
pub(super) fn sight(
    terrain: Option<Res<Terrain>>,
    robot: Query<&GlobalTransform, With<Player>>,
    creatures: Query<(&Creature, &GlobalTransform)>,
    mut seen: ResMut<Seen>,
) {
    let (Some(terrain), Ok(robot)) = (terrain, robot.single()) else {
        return;
    };
    let Some(life) = terrain.globe().life() else {
        return;
    };
    let near = creatures
        .iter()
        .map(|(c, at)| (c.species, at.translation()));
    for species in sighted(life, robot.translation(), near) {
        if !seen.has(species) {
            seen.mark(species);
        }
    }
}

/// Forgets every sighting when the player leaves the world.
pub(super) fn forget(mut seen: ResMut<Seen>) {
    *seen = Seen::default();
}
