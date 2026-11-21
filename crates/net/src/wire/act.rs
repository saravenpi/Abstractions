//! The hunting acts of a player and the events of a server: what travels on the reliable stream
//! about animals, spears, blows on robots, wounds, impacts, attacks, rocks and the things players put
//! on the ground.

use serde::{Deserialize, Serialize};

use super::field::{Attack, Blow, Impact, Opened, Rock, Wound};
use super::ground::Lying;
use super::hunt::{AnimalKey, Carcass, Hit, Kill, Spear, SpearKey};
use crate::log::PlayerId;

/// What a player tells the server about hunting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Act {
    /// A blow landed on an animal.
    Hit(Hit),
    /// A part of an animal broke.
    Broke {
        /// The animal.
        animal: AnimalKey,
        /// Index of the part.
        part: u8,
    },
    /// An animal was killed.
    Kill(Kill),
    /// A part was cut from a carcass.
    Carve {
        /// The animal it was.
        animal: AnimalKey,
        /// Index of the part.
        part: u8,
    },
    /// A spear stuck.
    Stick(Spear),
    /// A spear was pulled out.
    Pull(SpearKey),
    /// The player holds an item, or nothing.
    Held(Option<Vec<u8>>),
    /// A blow landed on the robot of another player.
    Blow(Blow),
    /// A wound opened on a part of an animal.
    Wound {
        /// The animal.
        animal: AnimalKey,
        /// The wound.
        wound: Wound,
    },
    /// A spear plugged the wound on a part of an animal.
    Plug {
        /// The animal.
        animal: AnimalKey,
        /// Index of the part.
        part: u8,
    },
    /// A swing or a throw landed, for the sound and the chips.
    Impact(Impact),
    /// An animal starts an attack.
    Attack(Attack),
    /// A rock was taken.
    Rock(Rock),
    /// The player puts a thing on the ground. The server gives it its id and tells everyone.
    Drop(Lying),
    /// The player asks to take a thing off the ground. The first to ask gets it.
    Take {
        /// The thing.
        id: u64,
        /// Most kilograms of a heap the player can carry; a tool is taken whole.
        mass: f32,
    },
}

/// Everything about hunting a player needs when joining.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HuntWorld {
    /// The carcasses that still lie.
    pub carcasses: Vec<Carcass>,
    /// The spears that still stand.
    pub spears: Vec<Spear>,
    /// What each player holds, for the players who hold something.
    pub held: Vec<(PlayerId, Vec<u8>)>,
    /// The wounds that still bleed or wait to be plugged.
    pub wounds: Vec<Opened>,
    /// The rocks taken, on every planet.
    pub rocks: Vec<Rock>,
}

/// What the server tells a player about the hunting of the others.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// The state of the hunt, after joining.
    World(HuntWorld),
    /// A player landed a blow.
    Hit {
        /// The player.
        by: PlayerId,
        /// The blow.
        hit: Hit,
    },
    /// A part broke.
    Broke {
        /// The animal.
        animal: AnimalKey,
        /// Index of the part.
        part: u8,
    },
    /// An animal was killed. Everyone gets it, the killer too.
    Killed(Carcass),
    /// A player cut a part from a carcass.
    Carved {
        /// The animal it was.
        animal: AnimalKey,
        /// Index of the part.
        part: u8,
        /// The player.
        by: PlayerId,
    },
    /// A spear stuck.
    Stuck(Spear),
    /// A player pulled a spear out.
    Pulled {
        /// The spear.
        key: SpearKey,
        /// The player.
        by: PlayerId,
    },
    /// A player holds an item, or nothing.
    Held {
        /// The player.
        by: PlayerId,
        /// The recipe bytes of the item.
        item: Option<Vec<u8>>,
    },
    /// A blow landed on the robot of the player who hears it.
    Blow {
        /// The player who struck.
        by: PlayerId,
        /// The blow.
        blow: Blow,
    },
    /// A wound opened.
    Wound {
        /// The animal.
        animal: AnimalKey,
        /// The wound.
        wound: Wound,
    },
    /// A wound was plugged.
    Plug {
        /// The animal.
        animal: AnimalKey,
        /// Index of the part.
        part: u8,
    },
    /// A swing or a throw landed.
    Impact(Impact),
    /// An animal starts an attack.
    Attack(Attack),
    /// A rock was taken.
    Rock(Rock),
    /// A thing lies on the ground: put down, grown by a heap that joined it, or left smaller by a
    /// take. Everyone gets it, the player who put it down too.
    Lies(Lying),
    /// A thing left the ground, taken whole.
    Gone {
        /// The thing.
        id: u64,
        /// The player who took it.
        by: PlayerId,
    },
    /// The thing the player asked for is theirs: the whole of it, or the part of a heap asked.
    Granted(Lying),
    /// The thing the player asked for is gone: another player took it first.
    Denied(u64),
    /// The ground could not take the drop with this tag, so the thing stays the player's.
    Kept(u32),
    /// Things that lie on the ground, after joining, a part of them in each message.
    Ground(Vec<Lying>),
}
