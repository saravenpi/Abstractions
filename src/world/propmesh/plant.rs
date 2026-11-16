//! The list of everything that can be placed on the ground, and what each kind needs from its surroundings.

use super::{BIG, PARTS};

/// Everything that can be placed on the ground. Declaration order is the index into the mesh tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plant {
    /// Round oak tree.
    Oak,
    /// Tall green pine.
    Pine,
    /// Pink blossom tree.
    Cherry,
    /// Pine with white snow on its tiers.
    SnowPine,
    /// Palm with a leaning trunk.
    Palm,
    /// Slim pale birch.
    Birch,
    /// Old broad oak with a thick trunk.
    GreatOak,
    /// Leafless grey tree.
    DeadTree,
    /// Clump of tall bamboo canes.
    Bamboo,
    /// Weeping willow with long hanging curtains of leaves.
    Willow,
    /// Saguaro cactus with two arms.
    Saguaro,
    /// Small round cactus with a flower.
    RoundCactus,
    /// Leafy bush.
    Bush,
    /// Bush with red berries.
    BerryBush,
    /// Bare brown shrub.
    DeadBush,
    /// Fallen log.
    Log,
    /// Grey rock.
    Rock,
    /// Pale ice-grey rock.
    IceRock,
    /// Orange sandstone rock.
    Sandstone,
    /// Smooth sandy-grey stones of the shallows.
    SmoothRock,
    /// Big dark boulder of the deep sea floor.
    Boulder,
    /// Weathered branch lying in the shallows.
    Driftwood,
    /// Broken rowing boat.
    Boat,
    /// Old anchor lying on its side.
    Anchor,
    /// Terracotta amphora.
    Amphora,
    /// Sunken wooden chest.
    Chest,
    /// Grass tuft.
    Grass,
    /// Tall reed with a brown head.
    Reed,
    /// Big leafy tropical plant.
    Jungle,
    /// Fern.
    Fern,
    /// Red-capped mushroom.
    Mushroom,
    /// Tall straw-colored grass.
    TallGrass,
    /// Fallen autumn leaves.
    Leaves,
    /// Seagrass blades.
    Seagrass,
    /// Orange starfish.
    Starfish,
    /// Small scallop shell.
    Shell,
    /// Ring of four mushrooms.
    MushroomRing,
    /// Pair of beach pebbles.
    Pebbles,
    /// Branching staghorn coral.
    BranchCoral,
    /// Round brain coral dome.
    BrainCoral,
    /// Flat sea fan.
    FanCoral,
    /// Cluster of tube corals.
    TubeCoral,
    /// Sea anemone with tentacles.
    Anemone,
    /// Barrel sponge.
    Sponge,
    /// Giant clam with a bright mantle.
    Clam,
    /// Tall kelp ribbon whose height is stored in the placement scale.
    Kelp,
    /// Lily pad, sometimes with a flower, floating on a pond.
    Lily,
    /// Cupped lotus pad with a tall flower or seed pod.
    Lotus,
    /// Cattail rising from the pond floor above the water.
    Cattail,
    /// A few specks of duckweed on the pond surface.
    Duckweed,
    /// Red poppy.
    Poppy,
    /// White daisy.
    Daisy,
    /// Yellow buttercup.
    Buttercup,
    /// Blue bell.
    Bluebell,
    /// Pink blossom flower.
    Pink,
    /// Bright tropical orchid.
    Orchid,
}

impl Plant {
    /// Returns true for plants made of solid parts, which are merged through the shared vertex cache.
    pub fn solid(self) -> bool {
        let i = self as usize;
        i < BIG || PARTS.contains(&i)
    }

    /// Returns true for plants of the pond surface, whose geometry follows the water level.
    pub fn pond(self) -> bool {
        matches!(
            self,
            Plant::Lily | Plant::Lotus | Plant::Cattail | Plant::Duckweed
        )
    }

    /// Returns true for sea life and pond plants that grow in dense patches.
    pub fn patchy(self) -> bool {
        use Plant::*;
        matches!(
            self,
            Seagrass
                | Starfish
                | Shell
                | BranchCoral
                | BrainCoral
                | FanCoral
                | TubeCoral
                | Anemone
                | Sponge
                | Clam
        )
    }

    /// Returns the water depth per unit of scale that keeps the plant fully submerged.
    pub fn draft(self) -> f32 {
        use Plant::*;
        match self {
            Seagrass | TubeCoral => 1.5,
            Starfish | Shell | Pebbles => 0.1,
            SmoothRock | Anemone | Clam | Chest => 0.9,
            Driftwood | Anchor => 0.6,
            Boulder => 1.9,
            Boat | Sponge | BranchCoral => 1.3,
            Amphora => 1.6,
            BrainCoral | FanCoral => 1.1,
            _ => 0.0,
        }
    }
}
