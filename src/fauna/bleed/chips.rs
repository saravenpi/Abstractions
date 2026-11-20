//! Chips: what a blow knocks off a hard outer layer it did not cut: horn, plate, shell, bone.

use super::open::stack;
use super::tune::{CHIP, CHIP_FAST, CHIP_SHARE, CHIPS, HARD};
use crate::craft::build::{Blow, Mode, strike};
use crate::craft::matter::Material;
use crate::life::Species;
use crate::life::blood::drop_volume;

/// Chips knocked off by a blow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chips {
    /// How many.
    pub count: usize,
    /// Linear RGB colour of the tissue.
    pub colour: [f32; 3],
    /// Speed they leave at, in metres per second.
    pub speed: f32,
}

/// Returns true when `material` is stiff enough to chip rather than give: over [`HARD`] gigapascals.
fn hard(material: &Material) -> bool {
    material.stiffness >= HARD
}

/// Returns the chips that `blow` knocks off part `index` of `species`: none when the part has no
/// hard outer layer or the blow cut through it, unless it `glanced` off along the
/// surface. They share [`CHIP_SHARE`] of the energy of the blow as motion, so a harder blow throws
/// more of them faster, up to [`CHIP_FAST`].
pub fn chips(species: &Species, index: usize, blow: &Blow, glanced: bool) -> Option<Chips> {
    let (layers, skin) = stack(species, index);
    let outer = layers.first().filter(|_| skin > 0)?;
    let cut = !glanced && blow.mode != Mode::Blunt && strike(blow, &layers).passed > 0;
    if !hard(&outer.material) || cut {
        return None;
    }
    let count = (blow.energy.max(0.0).sqrt() / 2.0).clamp(1.0, CHIPS as f32) as usize;
    let mass = outer.material.density * drop_volume(CHIP);
    let speed = (2.0 * CHIP_SHARE * blow.energy / (count as f32 * mass)).sqrt();
    let tissue = &species.tissues[usize::from(species.body.parts[index].tissue)];
    Some(Chips {
        count,
        colour: tissue.colour,
        speed: speed.min(CHIP_FAST),
    })
}
