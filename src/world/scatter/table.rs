//! The plant tables: which plants may grow on each surface of each biome, with their chance per try.

use Plant::*;

use crate::world::propmesh::Plant;
use crate::world::{Biome, Surface};

/// Boulders on bare rock.
const ROCKY: &[(Plant, f32)] = &[(Rock, 0.12)];

/// Ice-grey stones on snow.
const SNOWY: &[(Plant, f32)] = &[(IceRock, 0.06)];

/// Kelp forests, boulders and rare sunken things on the deep sea floor.
const DEEP: &[(Plant, f32)] = &[
    (Kelp, 0.25),
    (Boulder, 0.012),
    (Boat, 0.0002),
    (Anchor, 0.0003),
    (Amphora, 0.0005),
    (Chest, 0.0002),
];

/// Seagrass, shells, starfish, smooth rocks and driftwood on the pale shelf.
const SHALLOWS: &[(Plant, f32)] = &[
    (SmoothRock, 0.012),
    (Driftwood, 0.006),
    (Seagrass, 0.22),
    (Shell, 0.06),
    (Starfish, 0.03),
    (Pebbles, 0.02),
];

/// Corals, anemones, sponges and clams on the reef.
const REEF: &[(Plant, f32)] = &[
    (SmoothRock, 0.01),
    (BranchCoral, 0.035),
    (BrainCoral, 0.035),
    (FanCoral, 0.03),
    (TubeCoral, 0.03),
    (Anemone, 0.015),
    (Sponge, 0.012),
    (Clam, 0.006),
    (Seagrass, 0.03),
    (Starfish, 0.015),
];

/// Lilies, lotus, cattails, reeds, duckweed and a few stones on the floor of a pond.
const POND: &[(Plant, f32)] = &[
    (Lily, 0.12),
    (Lotus, 0.08),
    (Cattail, 0.05),
    (Reed, 0.05),
    (Duckweed, 0.1),
    (Pebbles, 0.02),
];

/// Only a few reeds in a frozen pond.
const FROZEN_POND: &[(Plant, f32)] = &[(Reed, 0.03)];

/// Plants and their chance per try, by biome then by surface in the order deep water, shallows, reef, pond, sand,
/// grass, rock, snow.
#[rustfmt::skip]
const TABLE: [[&[(Plant, f32)]; 8]; 6] = [
    [DEEP, SHALLOWS, REEF, POND, &[(Willow, 0.1), (Rock, 0.03), (Grass, 0.08), (Pebbles, 0.04)],
        &[(Willow, 0.15), (Oak, 0.08), (GreatOak, 0.015), (Birch, 0.04), (Bush, 0.06), (BerryBush, 0.02), (Rock, 0.01),
        (Grass, 0.35), (TallGrass, 0.12), (Poppy, 0.12), (Daisy, 0.12), (Buttercup, 0.1), (Bluebell, 0.1)],
        ROCKY, SNOWY],
    [DEEP, SHALLOWS, REEF, POND, &[(Willow, 0.1), (Rock, 0.03), (Fern, 0.05), (Pebbles, 0.03)],
        &[(Willow, 0.15), (Oak, 0.22), (Pine, 0.2), (Birch, 0.1), (GreatOak, 0.03), (Bush, 0.04), (BerryBush, 0.02),
        (Log, 0.03), (Rock, 0.01), (Grass, 0.25), (Fern, 0.12), (Mushroom, 0.04), (MushroomRing, 0.004), (Leaves, 0.1),
        (TallGrass, 0.05), (Daisy, 0.02)],
        ROCKY, SNOWY],
    [DEEP, SHALLOWS, REEF, POND, &[(Willow, 0.1), (Rock, 0.03), (Grass, 0.08), (Pebbles, 0.03)],
        &[(Willow, 0.15), (Cherry, 0.2), (Oak, 0.02), (Birch, 0.03), (Bush, 0.04), (BerryBush, 0.015), (Rock, 0.01),
        (Grass, 0.3), (TallGrass, 0.08), (Pink, 0.2), (Daisy, 0.12)],
        ROCKY, SNOWY],
    [DEEP, SHALLOWS, REEF, POND,
        &[(Saguaro, 0.05), (RoundCactus, 0.06), (DeadBush, 0.06), (DeadTree, 0.01), (Sandstone, 0.04),
        (Grass, 0.02), (Pebbles, 0.02)],
        &[(Saguaro, 0.04), (RoundCactus, 0.05), (DeadBush, 0.05), (DeadTree, 0.01), (Sandstone, 0.03),
        (Grass, 0.03)],
        &[(Sandstone, 0.12)], SNOWY],
    [DEEP, SHALLOWS, REEF, FROZEN_POND, &[(IceRock, 0.04), (Pebbles, 0.03)],
        &[(SnowPine, 0.28), (Pine, 0.08), (DeadTree, 0.03), (DeadBush, 0.04), (IceRock, 0.03),
        (Grass, 0.12), (Fern, 0.03)],
        &[(IceRock, 0.12)], &[(SnowPine, 0.2), (DeadTree, 0.02), (IceRock, 0.04)]],
    [DEEP, SHALLOWS, REEF, POND,
        &[(Willow, 0.1), (Palm, 0.14), (Rock, 0.02), (Reed, 0.2), (Orchid, 0.02), (Pebbles, 0.03)],
        &[(Willow, 0.15), (Palm, 0.12), (Bamboo, 0.06), (Bush, 0.06), (BerryBush, 0.02), (Rock, 0.01), (Jungle, 0.1),
        (Grass, 0.25), (TallGrass, 0.05), (Orchid, 0.12), (Poppy, 0.08), (Buttercup, 0.08), (Reed, 0.03)],
        ROCKY, SNOWY],
];

/// Returns the plants that may grow on a surface of a biome, with their chance per try. A surface the
/// table has no column for, such as a river bed, grows nothing.
pub fn plants_for(biome: Biome, surface: Surface) -> &'static [(Plant, f32)] {
    TABLE[biome as usize]
        .get(surface as usize)
        .copied()
        .unwrap_or_default()
}
