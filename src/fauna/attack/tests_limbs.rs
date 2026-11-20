//! Tests of broken parts: the bits, the limp and the energy it takes to break a part.

use super::fracture::breaks;
use super::limbs::Broken;
use crate::fauna::rig::planet;
use crate::life::body::PartKind;

/// A broken leg limps in proportion to the legs broken, and a whole body keeps its pace.
#[test]
fn broken_legs_limp() {
    let (_, _, life) = planet(true);
    let life = life.expect("the home planet lives");
    let walker = life
        .species
        .iter()
        .find(|s| s.body.legs.len() >= 2)
        .expect("a walker lives there");
    let body = &walker.body;
    assert_eq!(Broken::default().pace(body), 1.0);
    let bones = body.legs.iter().flat_map(|l| l.bones.iter());
    let all = bones.fold(Broken::default(), |b, i| b.with(usize::from(*i)));
    assert!((all.lame(body) - 1.0).abs() < 1e-6);
    assert!(all.pace(body) < 0.5);
}

/// A brittle part breaks under a blow of enough energy and not under a touch; a torso never breaks.
#[test]
fn a_hard_enough_blow_breaks_a_part() {
    let (_, _, life) = planet(true);
    let life = life.expect("the home planet lives");
    let mut broke = 0;
    for species in &life.species {
        for (index, part) in species.body.parts.iter().enumerate() {
            assert!(!breaks(species, index, 1e-6));
            broke += usize::from(breaks(species, index, 1e9));
            if part.kind == PartKind::Torso {
                assert!(!breaks(species, index, 1e9));
            }
        }
    }
    assert!(broke > 0);
}
