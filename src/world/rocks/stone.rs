//! The stones a planet's geology leaves on the ground, and which rock prop shows which stone: what a
//! rock looks like is what the robot breaks off it.

use serde::{Deserialize, Serialize};

use crate::universe::{NO_AIR, PlanetClass, PlanetRecord, Water};
use crate::world::propmesh::Plant;

/// A stone that can be picked up and worked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mineral {
    /// Dark volcanic rock, on every solid world: heavy, hard, and tough for a stone.
    Basalt,
    /// Volcanic glass: the sharpest edge there is, and the most brittle.
    Obsidian,
    /// Chert grown from silica in water: hard, and knaps to a lasting edge.
    Flint,
    /// Frozen water: light, soft and brittle.
    Ice,
}

impl Mineral {
    /// Every mineral, in a stable order.
    pub const ALL: [Mineral; 4] = [
        Mineral::Basalt,
        Mineral::Obsidian,
        Mineral::Flint,
        Mineral::Ice,
    ];

    /// Returns true when the planet of `record` has this stone on its ground.
    ///
    /// Basalt lies on every solid world. Obsidian needs lava cooled fast, so molten rock now or an
    /// atmosphere, the sign of a geology that still outgasses. Flint needs liquid water, or a sea of
    /// it once. Ice needs the cold.
    pub fn lies_on(self, record: &PlanetRecord) -> bool {
        let solid = record.class != PlanetClass::GasGiant;
        let wet = record.water == Water::Liquid;
        let active = record.class == PlanetClass::Lava || record.pressure >= NO_AIR;
        match self {
            Mineral::Basalt => solid,
            Mineral::Obsidian => solid && active,
            Mineral::Flint => solid && wet,
            Mineral::Ice => solid && record.water == Water::Ice,
        }
    }

    /// Returns the rock prop that shows this stone on land.
    pub fn plant(self) -> Plant {
        match self {
            Mineral::Basalt => Plant::Rock,
            Mineral::Obsidian => Plant::Obsidian,
            Mineral::Flint => Plant::Flint,
            Mineral::Ice => Plant::IceRock,
        }
    }
}

/// Returns the stone the rock prop `plant` is made of, the one its mesh shows, or nothing for a prop
/// the robot cannot break a stone off: sandstone crumbles to sand under a blow, and the smooth stones
/// and pebbles of the shallows have no body to aim at.
pub fn mineral_of(plant: Plant) -> Option<Mineral> {
    match plant {
        Plant::Rock | Plant::Boulder => Some(Mineral::Basalt),
        Plant::Obsidian => Some(Mineral::Obsidian),
        Plant::Flint => Some(Mineral::Flint),
        Plant::IceRock => Some(Mineral::Ice),
        _ => None,
    }
}
