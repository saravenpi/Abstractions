//! Plants that belong to their planet: the dials the climate and the world turn on each producer group.
//!
//! A producer species stands for one plant group, drawn by its growth architecture (its `archetype`):
//! 0 trunk and branches, 1 rosette, 2 mat, 3 vine, 4 bulb, 5 stalk and cap, 6 kelp, 7 crystal lattice.

mod colour;
#[cfg(test)]
pub(crate) mod fixture;
mod shape;
#[cfg(test)]
mod tests;

pub use colour::Paint;
pub use shape::{Form, productivity};

use super::{Guild, Life, Rung, Species, World};

/// Number of growth architectures.
pub const ARCHITECTURES: u8 = 8;

/// The architecture every producer of a planet of mats and films is drawn as.
pub const MAT: u8 = 2;

/// The kelp architecture, the one that grows under the liquid.
pub const KELP: u8 = 6;

/// Axial tilt in radians under which a planet has no seasons its plants follow.
const SEASONAL_TILT: f32 = 0.15;

/// Latitude in radians under which the year stays the same: the tropics.
const TROPICS: f32 = 0.3;

/// Where a plant group is in its year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    /// No seasons: leaves, flowers and fruit all year.
    Ever,
    /// Spring: in flower.
    Bloom,
    /// Summer: in full leaf.
    Green,
    /// Autumn: in fruit, and the leaves of the deciduous turn.
    Fruit,
    /// Winter: the deciduous stand bare.
    Dormant,
}

/// The look of one producer group on its planet at one stage of its year.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dials {
    /// Growth architecture, 0 to 7.
    pub architecture: u8,
    /// Size, leaves and defences.
    pub form: Form,
    /// Colours of the moment.
    pub paint: Paint,
}

/// Returns the stage of the year at `latitude` radians when the planet is `season` of the way round its
/// orbit (0 is the summer of the north) and its axis is tilted by `tilt` radians.
pub fn stage(season: f32, latitude: f32, tilt: f32) -> Stage {
    if tilt < SEASONAL_TILT || latitude.abs() < TROPICS {
        return Stage::Ever;
    }
    let south = if latitude < 0.0 { 0.5 } else { 0.0 };
    let local = (season + south).rem_euclid(1.0);
    let quarters = [Stage::Green, Stage::Fruit, Stage::Dormant, Stage::Bloom];
    quarters[(local * 4.0 + 0.5) as usize % 4]
}

/// Returns the producer species of `life`, the plant groups.
pub fn producers(life: &Life) -> impl Iterator<Item = &Species> {
    life.species
        .iter()
        .filter(|s| s.role.guild == Guild::Producer)
}

/// Returns the architecture `species` is drawn as on `life`: its own, or a mat on a planet of mats.
pub fn architecture(life: &Life, species: &Species) -> u8 {
    match life.rung {
        Rung::Mats => MAT,
        _ => species.archetype.min(ARCHITECTURES - 1),
    }
}

/// Returns the dials of producer `species` of `life` on `world`, in a wind of `wind` metres per second
/// and at `stage` of its year.
pub fn dials(life: &Life, species: &Species, world: &World, wind: f32, stage: Stage) -> Dials {
    let form = shape::form(life, species, world, wind);
    Dials {
        architecture: architecture(life, species),
        paint: colour::paint(life, species, &form, stage),
        form,
    }
}
