//! Which species the player has seen. An animal counts as seen once a drawn [`Creature`] of it
//! came within [`SIGHT`] metres of the robot; the plants of a living planet count as seen once
//! the robot stands on it. The list is kept in the world file and read back when the world opens.

use std::collections::HashSet;

use bevy::prelude::*;

use super::super::Creature;
use crate::condition::Condition;
use crate::life::{Guild, Life, SpeciesId};
use crate::player::Player;
use crate::saves::ActiveWorld;
use crate::world::Terrain;

/// Distance in metres within which a creature counts as seen.
pub const SIGHT: f32 = 40.0;

/// The species the player has seen in the world being played, on every planet.
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
/// species of each creature at `creatures` within `reach` metres of `robot`, which is [`SIGHT`] for
/// a whole head and less for a hurt one.
pub fn sighted<'a>(
    life: &'a Life,
    robot: Vec3,
    creatures: impl Iterator<Item = (SpeciesId, Vec3)> + 'a,
    reach: f32,
) -> impl Iterator<Item = SpeciesId> + 'a {
    let plants = life
        .species
        .iter()
        .filter(|s| s.role.guild == Guild::Producer)
        .map(|s| s.id);
    let near = creatures
        .filter(move |(_, at)| at.distance_squared(robot) <= reach * reach)
        .map(|(species, _)| species);
    plants.chain(near)
}

/// Marks what the robot on foot sees on its planet. It changes [`Seen`] only for a new species.
pub(super) fn sight(
    terrain: Option<Res<Terrain>>,
    robot: Query<&GlobalTransform, With<Player>>,
    creatures: Query<(&Creature, &GlobalTransform)>,
    mut seen: ResMut<Seen>,
    condition: Option<Res<Condition>>,
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
    let reach = SIGHT * condition.map_or(1.0, |condition| condition.scan());
    for species in sighted(life, robot.translation(), near, reach) {
        if !seen.has(species) {
            seen.mark(species);
        }
    }
}

/// Reads the seen species of the world that was just made active: none when its file has none.
pub(super) fn load(active: Option<Res<ActiveWorld>>, mut seen: ResMut<Seen>) {
    let Some(active) = active.filter(|active| active.is_added()) else {
        return;
    };
    *seen = Seen(active.0.save.seen.iter().copied().collect());
}

/// Writes the seen species into the active world whenever they change, so the next save keeps them.
pub(super) fn store(seen: Res<Seen>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| seen.is_changed()) else {
        return;
    };
    let mut saved: Vec<SpeciesId> = seen.0.iter().copied().collect();
    saved.sort();
    if active.0.save.seen != saved {
        active.0.save.seen = saved;
    }
}
