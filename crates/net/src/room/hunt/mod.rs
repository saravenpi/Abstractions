//! The hunt the room keeps: who hurt which animal, the carcasses of the kills, the spears
//! that stand and what each player holds.
//!
//! The room never simulates an animal. It keeps what the players say, stamps the player
//! itself, and tells the others. A carcass lasts its time and then goes with the spears
//! stuck in it, so the ledger clears itself.

mod act;
mod field;
mod ledger;
mod spear;
mod state;

use std::collections::HashMap;

use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{AnimalKey, Carcass, HuntWorld, Opened, Rock, Spear};

/// Seconds the room remembers a wound of an animal that did not die.
pub(super) const HURT_KEEP: f64 = 1800.0;

/// Most wounded animals the room remembers.
pub(super) const HURT_MAX: usize = 4096;

/// Most wounds the room keeps.
pub(super) const WOUNDS_MAX: usize = 2048;

/// Most taken rocks the room keeps. Rocks past it are not remembered.
pub(super) const ROCKS_MAX: usize = 65_536;

/// Most players remembered as having hurt one animal.
pub(super) const BY_MAX: usize = 8;

/// What the room knows of an animal that was hurt and is not dead.
#[derive(Debug)]
pub(super) struct Hurt {
    /// The parts broken, a bit for each part index.
    pub(super) broken: u64,
    /// Universe time of the last blow.
    pub(super) last: f64,
    /// The players who hurt it, with the universe time of their last blow.
    pub(super) by: Vec<(PlayerId, f64)>,
}

/// The hunt of a room.
#[derive(Debug, Default)]
pub(super) struct Hunting {
    /// The carcasses that lie, oldest first.
    pub(super) carcasses: Vec<Carcass>,
    /// The spears that stand, oldest first.
    pub(super) spears: Vec<Spear>,
    /// The animals that were hurt.
    pub(super) hurt: HashMap<AnimalKey, Hurt>,
    /// The wounds of animals that are not dead, oldest first.
    pub(super) wounds: Vec<Opened>,
    /// The mass taken of each rock, in grams, with the mass of the whole rock, by planet and id.
    pub(super) rocks: HashMap<(BodyId, u64), (u32, u32)>,
}

impl Hunting {
    /// Returns the hunt of a world that saved `kept`, within the limits of a room.
    pub(super) fn new(kept: HuntWorld) -> Self {
        let mut hunt = Self::default();
        for carcass in kept.carcasses {
            hunt.lay(carcass);
        }
        for spear in kept.spears {
            hunt.plant(spear);
        }
        hunt.wounds = kept.wounds;
        hunt.wounds.truncate(WOUNDS_MAX);
        for rock in kept.rocks.into_iter().take(ROCKS_MAX) {
            hunt.rocks
                .insert((rock.planet, rock.id), (rock.grams.min(rock.whole), rock.whole));
        }
        hunt
    }

    /// Returns what a player who joins is told, with what the players `held`.
    pub(super) fn world(&self, held: Vec<(PlayerId, Vec<u8>)>) -> HuntWorld {
        HuntWorld {
            carcasses: self.carcasses.clone(),
            spears: self.spears.clone(),
            held,
            wounds: self.wounds.clone(),
            rocks: self
                .rocks
                .iter()
                .map(|(&(planet, id), &(grams, whole))| Rock {
                    planet,
                    id,
                    grams,
                    whole,
                })
                .collect(),
        }
    }
}
