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
    /// Hexagonal silica crystal: hard and sharp, common on dry, barren and cold worlds.
    Quartz,
    /// Dense emerald nephrite: extraordinarily tough, found on island and warm wet worlds.
    Jade,
    /// Heavy banded hematite iron-oxide rock: dense and strong, on dry, volcanic and barren worlds.
    Ironstone,
    /// Vivid banded copper-green carbonate stone, on desert and warm or drier island worlds.
    Malachite,
    /// Bright sulfur-yellow volcanic crystal outcrop, on lava and hot worlds.
    Brimstone,
    /// Rare nickel-iron meteoritic stone, on cratered barren, ice and thin-air worlds.
    Siderite,
}

impl Mineral {
    /// Every mineral, in a stable order.
    pub const ALL: [Mineral; 10] = [
        Mineral::Basalt,
        Mineral::Obsidian,
        Mineral::Flint,
        Mineral::Ice,
        Mineral::Quartz,
        Mineral::Jade,
        Mineral::Ironstone,
        Mineral::Malachite,
        Mineral::Brimstone,
        Mineral::Siderite,
    ];

    /// Returns true when the planet of `record` has this stone on its ground.
    ///
    /// Basalt lies on every solid world. Obsidian needs lava cooled fast or an atmosphere; flint needs
    /// liquid water; ice needs the cold. Quartz, jade, ironstone, malachite, brimstone and siderite
    /// follow temperature, sea share, air and volcanism without changing the temperate reference set.
    pub fn lies_on(self, record: &PlanetRecord) -> bool {
        use PlanetClass::{Barren, Desert, GasGiant, Ice, Island, Lava, Ocean};
        let solid = record.class != GasGiant;
        let wet = record.water == Water::Liquid;
        let active = record.class == Lava || record.pressure >= NO_AIR;
        let seas = matches!(record.class, Ocean | Island);
        let dry = matches!(record.class, Desert | Barren | Ice);
        let hot = record.class == Lava || record.temperature > 330.0;
        match self {
            Mineral::Basalt => solid,
            Mineral::Obsidian => solid && active,
            Mineral::Flint => solid && wet,
            Mineral::Ice => solid && record.water == Water::Ice,
            Mineral::Quartz => solid && (dry || record.temperature < 280.0),
            Mineral::Jade => {
                solid && seas && wet && (record.ocean < 0.55 || record.temperature > 294.0)
            }
            Mineral::Ironstone => solid && matches!(record.class, Desert | Lava | Barren),
            Mineral::Malachite => {
                solid && (record.class == Desert || (record.class == Island && record.ocean < 0.55))
            }
            Mineral::Brimstone => solid && hot,
            Mineral::Siderite => {
                solid && (matches!(record.class, Barren | Ice) || record.pressure < 0.4)
            }
        }
    }

    /// Returns the rock prop that shows this stone on land.
    pub fn plant(self) -> Plant {
        match self {
            Mineral::Basalt => Plant::Rock,
            Mineral::Obsidian => Plant::Obsidian,
            Mineral::Flint => Plant::Flint,
            Mineral::Ice => Plant::IceRock,
            Mineral::Quartz => Plant::Quartz,
            Mineral::Jade => Plant::Jade,
            Mineral::Ironstone => Plant::Ironstone,
            Mineral::Malachite => Plant::Malachite,
            Mineral::Brimstone => Plant::Brimstone,
            Mineral::Siderite => Plant::Siderite,
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
        Plant::Quartz => Some(Mineral::Quartz),
        Plant::Jade => Some(Mineral::Jade),
        Plant::Ironstone => Some(Mineral::Ironstone),
        Plant::Malachite => Some(Mineral::Malachite),
        Plant::Brimstone => Some(Mineral::Brimstone),
        Plant::Siderite => Some(Mineral::Siderite),
        _ => None,
    }
}
