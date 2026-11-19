//! Growth architectures of producers, in the fixed order of the contract: 0 trunk and branches,
//! 1 rosette, 2 mat, 3 vine, 4 bulb, 5 stalk and cap, 6 kelp, 7 crystal lattice.

use super::super::chem::Setting;
use super::super::{Rung, Solvent, Structure};
use crate::universe::hash::Rng;

/// Trunk and branches.
const TRUNK: u8 = 0;
/// Rosette.
const ROSETTE: u8 = 1;
/// Mat.
const MAT: u8 = 2;
/// Vine.
const VINE: u8 = 3;
/// Bulb.
const BULB: u8 = 4;
/// Stalk and cap.
const CAP: u8 = 5;
/// Kelp.
const KELP: u8 = 6;
/// Crystal lattice.
const LATTICE: u8 = 7;

/// Returns the architectures the climate and rung of `setting` allow, the defining one first.
///
/// Mats grow everywhere life does. Soft life adds caps, bulbs and kelp where there is a liquid,
/// shelled life rosettes, and skeletal life trunks and vines, which the trunk then leads. Cold
/// worlds have no vines, dry worlds no kelp, and silica life grows crystal lattices.
fn allowed(setting: &Setting) -> Vec<u8> {
    let (world, chemistry) = (&setting.world, &setting.chemistry);
    let crystal =
        chemistry.structure == Structure::Silica || chemistry.solvent == Solvent::Silicate;
    let rung = setting.rung;
    let mut list = Vec::with_capacity(8);
    list.extend((rung >= Rung::Skeletal && !crystal).then_some(TRUNK));
    list.extend(crystal.then_some(LATTICE));
    list.push(MAT);
    list.extend((rung >= Rung::Soft).then_some(CAP));
    list.extend((rung >= Rung::Soft).then_some(BULB));
    list.extend((rung >= Rung::Soft && world.ocean > 0.0).then_some(KELP));
    list.extend((rung >= Rung::Shelled).then_some(ROSETTE));
    list.extend((rung >= Rung::Skeletal && world.temperature > 250.0).then_some(VINE));
    list
}

/// Returns the architecture of each of `count` producers on the planet of `setting`.
///
/// The defining architecture always comes first; the others follow in a seeded order and repeat
/// only once every allowed one is used.
pub(super) fn architectures(count: usize, setting: &Setting, rng: &mut Rng) -> Vec<u8> {
    let mut list = allowed(setting);
    for i in (2..list.len()).rev() {
        let j = 1 + rng.below(i as u32) as usize;
        list.swap(i, j);
    }
    (0..count).map(|i| list[i % list.len()]).collect()
}
