//! Tests for armor defense tiers, Aegis shield capacities, body coverage, and pack equip/unequip.

use super::{Armor, ArmorSlot};
use crate::condition::Part;
use crate::craft::matter::Mineral;
use crate::craft::pack::{Pack, SlotEntry};

/// Builds a fresh test armor piece for `slot` and `mineral`.
fn piece(slot: ArmorSlot, mineral: Mineral, shielded: bool) -> Armor {
    Armor {
        id: 0,
        slot,
        mineral,
        shielded,
        wear: 0.0,
    }
}

/// Defense scales strictly across all six mineral tiers, with Lonsdaleite at ~82 percent.
#[test]
fn armor_defense_scales_across_all_six_mineral_tiers() {
    let basalt = piece(ArmorSlot::Chest, Mineral::Basalt, false).defense();
    let ironstone = piece(ArmorSlot::Chest, Mineral::Ironstone, false).defense();
    let siderite = piece(ArmorSlot::Chest, Mineral::Siderite, false).defense();
    let chromite = piece(ArmorSlot::Chest, Mineral::Chromite, false).defense();
    let corundum = piece(ArmorSlot::Chest, Mineral::Corundum, false).defense();
    let wolframite = piece(ArmorSlot::Chest, Mineral::Wolframite, false).defense();
    let moissanite = piece(ArmorSlot::Chest, Mineral::Moissanite, false).defense();
    let osmium = piece(ArmorSlot::Chest, Mineral::Osmium, false).defense();
    let lonsdaleite = piece(ArmorSlot::Chest, Mineral::Lonsdaleite, false).defense();
    assert!(lonsdaleite > osmium && osmium > moissanite);
    assert!(moissanite > wolframite && wolframite > corundum);
    assert!(corundum > chromite && chromite > siderite);
    assert!(siderite > ironstone && ironstone > basalt);
    assert!((0.80..=0.84).contains(&lonsdaleite));
}

/// Full Lonsdaleite Aegis grants ~1,400 J shield, outscaling lower-tier shielded sets.
#[test]
fn aegis_shield_scales_up_to_lonsdaleite_apex() {
    let set_shield = |m: Mineral| -> f32 {
        ArmorSlot::ALL
            .into_iter()
            .map(|slot| piece(slot, m, true).shield_joules())
            .sum()
    };
    let copper = set_shield(Mineral::Copper);
    let cobalt = set_shield(Mineral::Cobaltite);
    let wolfram = set_shield(Mineral::Wolframite);
    let osmium = set_shield(Mineral::Osmium);
    let lons = set_shield(Mineral::Lonsdaleite);
    assert_eq!(
        piece(ArmorSlot::Chest, Mineral::Lonsdaleite, false).shield_joules(),
        0.0
    );
    assert!(lons > osmium && osmium > wolfram && wolfram > cobalt && cobalt > copper);
    assert!(
        (lons - 1396.0).abs() < 30.0,
        "lonsdaleite shield was {lons}"
    );
}

/// Every one of the ten robot parts is covered by exactly one armor slot.
#[test]
fn every_body_part_is_covered_by_exactly_one_slot() {
    for part in Part::ALL {
        let hits = ArmorSlot::ALL
            .into_iter()
            .filter(|slot| slot.covers(part))
            .count();
        assert_eq!(hits, 1, "{part:?}");
    }
}

/// Adding, equipping, swapping, and unequipping armor updates pack defense and shield totals.
#[test]
fn pack_equip_swap_and_unequip_armor_updates_stats() {
    let mut pack = Pack::default();
    let a = pack
        .add_armor(ArmorSlot::Chest, Mineral::Basalt, false)
        .unwrap();
    let b = pack
        .add_armor(ArmorSlot::Chest, Mineral::Lonsdaleite, true)
        .unwrap();
    let slot_a = pack
        .grid
        .iter()
        .position(|s| *s == Some(SlotEntry::Armor(a)))
        .unwrap();
    let slot_b = pack
        .grid
        .iter()
        .position(|s| *s == Some(SlotEntry::Armor(b)))
        .unwrap();
    assert!(pack.equip_armor_from_slot(slot_a));
    assert_eq!(pack.total_shield_max(), 0.0);
    let low_def = pack.defense_for(Part::Torso);
    assert!(pack.equip_armor_from_slot(slot_b));
    assert_eq!(pack.grid[slot_b], Some(SlotEntry::Armor(a)));
    assert!(pack.defense_for(Part::Torso) > low_def * 3.0);
    assert!(pack.total_shield_max() > 500.0);
    assert!(pack.unequip_armor(ArmorSlot::Chest));
    assert_eq!(pack.defense_for(Part::Torso), 0.0);
    assert_eq!(pack.total_shield_max(), 0.0);
}
