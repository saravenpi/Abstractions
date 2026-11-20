//! The species of one producer group: kin that share its growth architecture and its climate, each with
//! its own size, proportions, lean, branching, leaf count and tint, so that a stand is never made of copies.

use crate::life::Species;
use crate::universe::hash::{Rng, child};

/// Child of the planet seed the kinds of a group are drawn from, plus the index of the group.
const STREAM: u64 = 0x4B49_4E00;

/// Most species one producer group shows.
pub const MOST: u8 = 3;

/// Abundance of each species of a group by rank, the commonest first, before they are shared out.
const ABUNDANCE: [f32; MOST as usize] = [1.0, 0.55, 0.3];

/// What sets one species of a producer group apart from its kin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kind {
    /// Rank in its group, 0 the commonest.
    pub rank: u8,
    /// Share of the plants of its group that are of this species, the shares of a group summing to 1.
    pub share: f32,
    /// Where in the patch noise of its group it grows best, 0 to 1, so kin fill different stands.
    pub niche: f32,
    /// Height against the typical height of its group, 0.6 to 1.5.
    pub size: f32,
    /// Width against height, 0.65 (slender) to 1.45 (broad).
    pub spread: f32,
    /// Lean of its main stem from the vertical in radians, 0 to 0.3.
    pub lean: f32,
    /// Branches, shoots or shards, 2 to 5.
    pub branches: u8,
    /// Leaves, blades or fronds per cluster, 3 to 9.
    pub leaves: u8,
    /// Shift of its leaf colour within the palette of the planet, -1 (deep and cool) to 1 (pale and warm).
    pub tint: f32,
    /// How far its leaves and tips hang, 0 (upright) to 1 (weeping).
    pub droop: f32,
}

impl Kind {
    /// The one species of a group drawn without kin: middle of every range.
    pub const PLAIN: Self = Self {
        rank: 0,
        share: 1.0,
        niche: 0.5,
        size: 1.0,
        spread: 1.0,
        lean: 0.0,
        branches: 3,
        leaves: 5,
        tint: 0.0,
        droop: 0.0,
    };
}

/// Returns species `rank` of a group of `count`, drawn from `rng`, with its share before normalising.
fn draw(rng: &mut Rng, rank: u8, count: u8) -> Kind {
    let mut unit = || rng.unit() as f32;
    let slot = (f32::from(rank) + 0.5) / f32::from(count);
    Kind {
        rank,
        share: ABUNDANCE[usize::from(rank)],
        niche: (slot + 0.15 * (unit() - 0.5)).clamp(0.0, 1.0),
        size: 0.6 + 0.9 * unit(),
        spread: 0.65 + 0.8 * unit(),
        lean: 0.3 * unit() * unit(),
        branches: 2 + (unit() * 3.99) as u8,
        leaves: 3 + (unit() * 6.99) as u8,
        tint: 2.0 * unit() - 1.0,
        droop: unit() * unit(),
    }
}

/// Returns the species of producer `species` on a planet where it grows with `productivity` from 0 to 1:
/// two, or three where it does well, each seeded from the planet and the group so every visit agrees.
pub fn kinds(species: &Species, productivity: f32) -> Vec<Kind> {
    let count = if productivity > 0.3 { MOST } else { MOST - 1 };
    let seed = child(species.id.planet, STREAM + u64::from(species.id.index));
    let mut rng = Rng::new(seed);
    let mut all: Vec<Kind> = (0..count).map(|r| draw(&mut rng, r, count)).collect();
    let total: f32 = all.iter().map(|k| k.share).sum();
    all.iter_mut().for_each(|k| k.share /= total);
    all
}
