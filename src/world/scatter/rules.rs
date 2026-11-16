//! Table rules shared by the scatter tests.

use super::table::plants_for;
use super::*;
use crate::world::Biome;
use crate::world::propmesh::{Plant, TREES};
use Plant::*;

/// Returns true for things that only belong on the sea floor.
pub fn underwater(p: Plant) -> bool {
    matches!(
        p,
        SmoothRock
            | Boulder
            | Driftwood
            | Boat
            | Anchor
            | Amphora
            | Chest
            | Seagrass
            | Starfish
            | Shell
            | BranchCoral
            | BrainCoral
            | FanCoral
            | TubeCoral
            | Anemone
            | Sponge
            | Clam
            | Kelp
    )
}

/// Returns true for the corals and sponges that only grow on a reef.
fn coral(p: Plant) -> bool {
    (BranchCoral as usize..=Clam as usize).contains(&(p as usize))
}

/// Sea floor tables hold only sea life, coral is reef-only and kelp is deep-only.
pub fn check_sea(b: Biome, s: Surface) {
    let table = plants_for(b, s);
    assert!(!table.is_empty());
    for &(p, _) in table {
        assert!(underwater(p) || p == Pebbles, "{p:?} on {s:?}");
        assert!((p as usize) >= TREES);
        assert!(!coral(p) || s == Surface::Reef);
        assert!(p != Kelp || s == Surface::Water);
    }
    assert!(
        table
            .iter()
            .any(|e| e.0 == [Kelp, Seagrass, BranchCoral][s as usize])
    );
}

/// Pond tables hold only pond plants, reeds and stones, no flowers in the taiga, and nothing from the sea.
pub fn check_pond(b: Biome) {
    let table = plants_for(b, Surface::Pond);
    assert!(!table.is_empty());
    for &(p, _) in table {
        assert!(p.pond() || matches!(p, Reed | Pebbles), "{p:?} on a pond");
        assert!(b != Biome::Taiga || p == Reed);
    }
    for s in [Surface::Water, Surface::Shallows, Surface::Reef] {
        assert!(plants_for(b, s).iter().all(|e| !e.0.pond()));
    }
}
