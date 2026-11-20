//! The shelf of sown floras: the flora of a planet at one stage of its year, sown once and handed to every
//! chunk that asks, so a chunk neither reads the wind of its planet nor looks up its pages again.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{PoisonError, RwLock};

use super::grow::Flora;
use crate::life::flora::{Stage, producers};
use crate::life::{Life, Rung, Species};
use crate::universe::hash::mix;
use crate::world::globe::Globe;
use crate::world::propmesh::grown::edition;

/// Floras the shelf keeps; past that it drops the oldest.
const KEPT: usize = 16;

/// Every flora kept, with its key, the newest last.
static SHELF: RwLock<Vec<(u64, Flora)>> = RwLock::new(Vec::new());

/// Returns `seed` stirred with `value`.
fn fold(seed: u64, value: u64) -> u64 {
    mix(seed ^ value.wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

/// Returns a hash of the relations of `life`, which give its plants flowers and fruit.
fn ties(life: &Life) -> u64 {
    let mut hasher = DefaultHasher::new();
    life.relations.hash(&mut hasher);
    hasher.finish()
}

/// Returns the fingerprint of producer `s`: who it is, its architecture, its climate and its tissues.
fn group(s: &Species) -> impl Iterator<Item = u64> + '_ {
    let t = &s.tolerance;
    let heat = [t.cold, t.heat, t.dry, t.wet].map(|v| u64::from(v.to_bits()));
    let tissues = s.tissues.iter().flat_map(|k| {
        let look = [k.colour[0], k.colour[1], k.colour[2], k.hardness];
        look.map(|v| u64::from(v.to_bits()))
    });
    let who = [s.id.planet, u64::from(s.id.index), u64::from(s.archetype)];
    who.into_iter().chain(heat).chain(tissues)
}

/// Returns the key of the flora of `life` on `globe` at `stage`: the planet, the producers with their
/// climate and tissues, their ties to animals, the stage of the year and the edition of the book, whose
/// pages the flora names.
pub fn key(life: &Life, globe: &Globe, stage: Stage) -> u64 {
    let record = globe.record();
    let planet = [
        globe.seed(),
        globe.radius().to_bits(),
        globe.gravity().to_bits(),
        record.flux.to_bits(),
        record.pressure.to_bits(),
        record.temperature.to_bits(),
        stage as u64,
        edition() as u64,
        u64::from(life.rung == Rung::Mats),
        ties(life),
    ];
    let pigment = life.chemistry.pigment.map(|c| u64::from(c.to_bits()));
    let all = planet
        .into_iter()
        .chain(pigment)
        .chain(producers(life).flat_map(group));
    all.fold(0x0046_4C4F_5241, fold)
}

/// Returns the flora kept under `key`, or sows it with `sow` and keeps it.
pub fn kept(key: u64, sow: impl FnOnce() -> Flora) -> Flora {
    let shelf = SHELF.read().unwrap_or_else(PoisonError::into_inner);
    if let Some((_, flora)) = shelf.iter().find(|(k, _)| *k == key) {
        return flora.clone();
    }
    drop(shelf);
    let flora = sow();
    let mut shelf = SHELF.write().unwrap_or_else(PoisonError::into_inner);
    if shelf.len() >= KEPT {
        shelf.remove(0);
    }
    shelf.push((key, flora.clone()));
    flora
}
