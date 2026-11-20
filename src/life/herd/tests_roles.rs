//! Tests of herd roles and the kill ledger: the lookout is one stable adult, and a killed member is
//! gone until its recovery is over.

use bevy::math::DVec3;

use super::{DEATHS, Herd, HerdState, Ledger, lookout};
use crate::life::hunt::RECOVERY;

/// Returns a still herd of identity `id` with `base` slots, the first `young` of them young.
fn herd(id: u64, base: u8, young: u8) -> Herd {
    Herd {
        id,
        species: 0,
        count: base,
        base,
        young,
        centre: DVec3::Y,
        heading: DVec3::X,
        speed: 0.0,
        state: HerdState::Graze,
        home: DVec3::Y,
        spacing: 1.0,
        deaths: [None; DEATHS],
    }
}

/// The lookout is an adult, the same at every call, and not the same slot in every herd.
#[test]
fn the_lookout_is_a_stable_adult() {
    let mut slots = Vec::new();
    for id in 0..40u64 {
        let h = herd(id, 12, 3);
        let watch = lookout(&h);
        assert!((3..12).contains(&watch), "slot {watch}");
        assert_eq!(watch, lookout(&h));
        slots.push(watch);
    }
    slots.dedup();
    assert!(slots.len() > 1, "every herd picked slot {}", slots[0]);
}

/// A herd of young only keeps slot 0 as its lookout.
#[test]
fn young_herds_watch_from_slot_zero() {
    let h = herd(1, 2, 2);
    assert_eq!(lookout(&h), 0);
}

/// A killed member is dead from its death until death plus recovery, and the ledger then forgets it.
#[test]
fn the_ledger_clears_itself() {
    let mut ledger = Ledger::default();
    ledger.kill(7, 2, 100.0);
    ledger.kill(7, 2, 100.0);
    assert_eq!(ledger.kills().len(), 1);
    assert!(!ledger.dead(7, 2, 99.0));
    assert!(ledger.dead(7, 2, 100.0));
    assert!(!ledger.dead(7, 3, 100.0));
    assert!(ledger.dead(7, 2, 100.0 + RECOVERY - 1.0));
    assert!(!ledger.dead(7, 2, 100.0 + RECOVERY));
    ledger.prune(100.0 + RECOVERY);
    assert!(ledger.kills().is_empty());
}
