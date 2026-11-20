//! Tests of when a blow opens an animal: a bruise makes no wound, a cut that gets through does.

use super::chips::chips;
use super::open::opened;
use crate::craft::build::{Blow, Mode};
use crate::fauna::rig::planet;
use crate::life::body::PartKind;
use crate::life::{Life, Plan};

/// Returns the life of the home planet.
fn home() -> Life {
    planet(true).2.expect("the home planet lives")
}

/// Returns a blow of `energy` joules that meets the target as `mode`.
fn blow(mode: Mode, energy: f32) -> Blow {
    Blow {
        energy,
        mode,
        width: 0.05,
        area: 0.01,
        hardness: 9.9,
    }
}

/// A blunt blow opens no part of any species, however hard it hits: it bruises or breaks a shell and
/// the flesh under it is not cut.
#[test]
fn a_blunt_blow_opens_nothing() {
    for species in &home().species {
        for index in 0..species.body.parts.len() {
            assert_eq!(opened(species, index, &blow(Mode::Blunt, 1.0e5)), None);
        }
    }
}

/// An edge with the energy to get through opens the trunk to a major vessel, and a cut that went
/// deeper had more energy: the depth never falls as the energy grows.
#[test]
fn a_cut_through_opens_a_wound_that_deepens_with_energy() {
    let life = home();
    let mut trunks = 0;
    for species in life.species.iter().filter(|s| s.plan == Plan::Skeletal) {
        for (index, part) in species.body.parts.iter().enumerate() {
            let depths: Vec<f32> = [1.0, 10.0, 100.0, 1.0e3, 1.0e4, 1.0e5]
                .iter()
                .filter_map(|e| opened(species, index, &blow(Mode::Slash, *e)))
                .map(|o| o.depth)
                .collect();
            assert!(depths.windows(2).all(|w| w[0] <= w[1] + 1e-9));
            let deep = opened(species, index, &blow(Mode::Slash, 1.0e5));
            if let Some(open) = deep.filter(|_| part.kind == PartKind::Torso) {
                trunks += 1;
                assert!(open.major && open.depth > 0.0 && open.width == 0.05);
            }
        }
    }
    assert!(trunks > 0);
}

/// A hard outer layer that the blow did not cut throws chips, and one it cut through throws none.
#[test]
fn chips_come_off_a_layer_that_was_not_cut() {
    let life = home();
    let mut found = false;
    for species in &life.species {
        for index in 0..species.body.parts.len() {
            if let Some(c) = chips(species, index, &blow(Mode::Blunt, 50.0), false) {
                found = true;
                assert!(c.count >= 1 && c.speed > 0.0 && c.speed <= super::tune::CHIP_FAST);
                assert_eq!(
                    chips(species, index, &blow(Mode::Slash, 1.0e5), false),
                    None
                );
                assert!(chips(species, index, &blow(Mode::Slash, 1.0e5), true).is_some());
            }
        }
    }
    assert!(found);
}
