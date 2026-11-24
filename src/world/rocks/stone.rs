//! The stones a planet's geology leaves on the surface and underground, and which rock prop
//! shows each surface stone.

use serde::{Deserialize, Serialize};

use crate::universe::{NO_AIR, PlanetClass, PlanetRecord, Water};
use crate::world::propmesh::Plant;

/// A stone or ore mineral that can be quarried, dug underground, worked, or loaded as shot.
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
    /// Native orange-red metallic copper vein mined underground on temperate, island and desert worlds.
    Copper,
    /// Brassy iron-sulfide crystal mined underground on temperate, desert and lava worlds.
    Pyrite,
    /// Heavy silver-grey lead-silver ore mined deep on temperate, barren and cold worlds.
    Galena,
    /// Deep cobalt-blue alloy ore mined deep on icy, barren and oceanic worlds.
    Cobaltite,
    /// Vivid crimson mercury-sulfide crystal mined on volcanic and hot arid worlds.
    Cinnabar,
    /// Deep violet crystal geode mined deep on wet temperate, island and ocean worlds.
    Amethyst,
    /// Dark matte titanium-iron ore mined deep on barren, volcanic and desert worlds.
    Ilmenite,
    /// Pale icy-cyan fluoride crystal mined under ice worlds.
    Cryolite,
}

impl Mineral {
    /// The ten surface stones represented by visible rock outcrops, in stable order.
    pub const SURFACE: [Mineral; 10] = [
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

    /// Every mineral, surface rocks first then underground ores, in a stable order.
    pub const ALL: [Mineral; 18] = [
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
        Mineral::Copper,
        Mineral::Pyrite,
        Mineral::Galena,
        Mineral::Cobaltite,
        Mineral::Cinnabar,
        Mineral::Amethyst,
        Mineral::Ilmenite,
        Mineral::Cryolite,
    ];

    /// Returns true when the planet of `record` has this stone exposed on its surface.
    pub fn lies_on(self, record: &PlanetRecord) -> bool {
        use PlanetClass::{Barren, Desert, GasGiant, Ice, Island, Lava, Ocean};
        let solid = record.class != GasGiant;
        let wet = record.water == Water::Liquid;
        let active = record.class == Lava || record.pressure >= NO_AIR;
        let dry = matches!(record.class, Desert | Barren | Ice);
        let hot = record.class == Lava || record.temperature > 330.0;
        match self {
            Mineral::Basalt => solid,
            Mineral::Obsidian => solid && active,
            Mineral::Flint => solid && wet,
            Mineral::Ice => solid && record.water == Water::Ice,
            Mineral::Quartz => solid && (dry || record.temperature < 280.0),
            Mineral::Jade => {
                solid
                    && matches!(record.class, Ocean | Island)
                    && wet
                    && (record.ocean < 0.55 || record.temperature > 294.0)
            }
            Mineral::Ironstone => solid && matches!(record.class, Desert | Lava | Barren),
            Mineral::Malachite => {
                solid && (record.class == Desert || (record.class == Island && record.ocean < 0.55))
            }
            Mineral::Brimstone => solid && hot,
            Mineral::Siderite => {
                solid && (matches!(record.class, Barren | Ice) || record.pressure < 0.4)
            }
            _ => false,
        }
    }

    /// Returns the surface rock prop for this stone when it crops out on the surface.
    pub fn surface_plant(self) -> Option<Plant> {
        match self {
            Mineral::Basalt => Some(Plant::Rock),
            Mineral::Obsidian => Some(Plant::Obsidian),
            Mineral::Flint => Some(Plant::Flint),
            Mineral::Ice => Some(Plant::IceRock),
            Mineral::Quartz => Some(Plant::Quartz),
            Mineral::Jade => Some(Plant::Jade),
            Mineral::Ironstone => Some(Plant::Ironstone),
            Mineral::Malachite => Some(Plant::Malachite),
            Mineral::Brimstone => Some(Plant::Brimstone),
            Mineral::Siderite => Some(Plant::Siderite),
            _ => None,
        }
    }

    /// Returns the rock prop that shows this stone on land, falling back to generic rock for deep ores.
    pub fn plant(self) -> Plant {
        self.surface_plant().unwrap_or(Plant::Rock)
    }
}

/// Returns the stone the rock prop `plant` is made of, or nothing for a prop the robot cannot break a
/// stone off.
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
