//! The colours of a producer group: leaves from the pigment of the planet, flowers only for a pollinator,
//! fruit only for a seed carrier, and the turn of the year.

use super::{Form, Stage};
use crate::life::{Life, Relation, Species};

/// Linear RGB of the stem of a group without tissues: dark bark.
const BARK: [f32; 3] = [0.06, 0.035, 0.018];

/// Linear RGB the waxy skin of dry places pales leaves towards.
const BLOOM: [f32; 3] = [0.32, 0.36, 0.34];

/// Linear RGB petal colours a pollinated group draws from, bright enough to read against any leaf.
const PETALS: [[f32; 3]; 6] = [
    [0.85, 0.05, 0.25],
    [0.95, 0.7, 0.05],
    [0.4, 0.1, 0.9],
    [0.95, 0.95, 0.92],
    [0.95, 0.3, 0.02],
    [0.1, 0.55, 0.95],
];

/// Linear RGB fruit colours a group with a seed carrier draws from.
const FRUIT: [[f32; 3]; 4] = [
    [0.7, 0.03, 0.03],
    [0.9, 0.4, 0.02],
    [0.25, 0.02, 0.45],
    [0.9, 0.8, 0.1],
];

/// The colours of a producer group at one stage of its year, linear RGB.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paint {
    /// Leaves, or none while a deciduous group stands bare.
    pub leaves: Option<[f32; 3]>,
    /// Stems, trunks and bodies.
    pub stem: [f32; 3],
    /// Flowers, only where a pollinator lives and in its season.
    pub flowers: Option<[f32; 3]>,
    /// Fruit, only where a seed carrier lives and in its season.
    pub fruit: Option<[f32; 3]>,
}

/// Returns a well stirred copy of `x`, the finaliser of SplitMix64: `universe::hash` is private to the universe.
fn mix(x: u64) -> u64 {
    let x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Returns the colour `a` moved a share `t` of the way to `b`.
fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

/// Returns the leaf colour of a group of `form` under a `pigment`: darker as needles, paler under wax.
fn leaf(pigment: [f32; 3], form: &Form) -> [f32; 3] {
    let tone = if form.needles { 0.7 } else { 1.0 };
    let base = pigment.map(|c| c * tone);
    if form.wax {
        lerp(base, BLOOM, 0.3)
    } else {
        base
    }
}

/// Returns the turned colour of a leaf of colour `c` in autumn: its brightest channel goes to red and gold.
fn autumn(c: [f32; 3]) -> [f32; 3] {
    let peak = c[0].max(c[1]).max(c[2]);
    lerp(c, [peak * 1.2, peak * 0.5, c[2] * 0.3], 0.8)
}

/// Returns the colour of `set` that `seed` picks, passing over those too close to `leaf` to read against it.
fn pick(seed: u64, leaf: [f32; 3], set: &[[f32; 3]]) -> [f32; 3] {
    let far = |c: &[f32; 3]| (0..3).map(|i| (c[i] - leaf[i]).abs()).sum::<f32>() > 0.4;
    let start = (seed % set.len() as u64) as usize;
    let turn = (0..set.len()).map(|k| set[(start + k) % set.len()]);
    turn.clone().find(far).unwrap_or(set[start])
}

/// Returns true when `relation` ties producer `plant` to an animal: pollination for `flowers`, or sowing.
fn tied(relation: &Relation, plant: u16, flowers: bool) -> bool {
    match *relation {
        Relation::Pollinates { plant: p, .. } => flowers && p == plant,
        Relation::Sows { plant: p, .. } => !flowers && p == plant,
        _ => false,
    }
}

/// Returns the colours of producer `species` of `life`, of `form`, at `stage` of its year.
pub fn paint(life: &Life, species: &Species, form: &Form, stage: Stage) -> Paint {
    let index = species.id.index;
    let seed = mix(species.id.planet ^ u64::from(index).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let has = |flowers: bool| life.relations.iter().any(|r| tied(r, index, flowers));
    let green = leaf(life.chemistry.pigment, form);
    let leaves = match stage {
        Stage::Dormant if form.deciduous => None,
        Stage::Fruit if form.deciduous => Some(autumn(green)),
        _ => Some(green),
    };
    let bloom = matches!(stage, Stage::Ever | Stage::Bloom) && has(true);
    let ripe = matches!(stage, Stage::Ever | Stage::Fruit) && has(false);
    Paint {
        leaves,
        stem: species.tissues.first().map_or(BARK, |t| t.colour),
        flowers: bloom.then(|| pick(seed, green, &PETALS)),
        fruit: ripe.then(|| pick(seed >> 17, green, &FRUIT)),
    }
}
