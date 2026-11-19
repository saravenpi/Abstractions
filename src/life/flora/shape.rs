//! The form of a producer group: how tall, how layered, how leafy and how armed its climate and its world
//! make it.

use crate::life::{Life, Reaction, Solvent, Species, TissueKind, World};

/// Typical height in metres of each architecture at the gravity of Earth in still air, in architecture order.
const BASE: [f32; 8] = [7.0, 0.9, 0.06, 2.2, 0.6, 3.5, 5.0, 2.6];

/// Surface gravity of Earth in metres per second squared, the gravity the base heights are for.
const EARTH_G: f32 = 9.81;

/// Wind in metres per second that bends a plant in air of the home pressure as much as it stands.
const GALE: f32 = 8.0;

/// Kelvin below the best temperature of the solvent under which a group lives in the cold.
const COLD: f32 = 20.0;

/// Kelvin above the best temperature of the solvent over which a group lives in the heat.
const HOT: f32 = 12.0;

/// Moisture under which a group lives in a dry place.
const DRY: f32 = 0.35;

/// Hardness of an accent tissue from which it shows as thorns.
const THORN: f32 = 4.0;

/// Size, leaves and defences of a producer group.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Form {
    /// Typical adult height in metres.
    pub height: f32,
    /// Canopy layers, 1 (ground) to 4 (ground, shrub, canopy and emergent).
    pub layers: u8,
    /// Leaf size, about 0.15 (scales) to 1.4 (broad leaves).
    pub leaf: f32,
    /// Needles instead of leaves, in the cold.
    pub needles: bool,
    /// A waxy pale skin, in dry places.
    pub wax: bool,
    /// Spines, in the driest and hottest places.
    pub spines: bool,
    /// A swollen water storing body, in the driest places.
    pub store: bool,
    /// Thorns from a hard accent tissue.
    pub thorns: bool,
    /// Resin beads from a resin tissue.
    pub resin: bool,
    /// Sheds its leaves in winter.
    pub deciduous: bool,
}

/// Returns the temperature in kelvin at which life built on `solvent` grows best.
fn optimum(solvent: Solvent) -> f32 {
    match solvent {
        Solvent::Water => 293.0,
        Solvent::Ammonia => 225.0,
        Solvent::Hydrocarbon => 100.0,
        Solvent::Silicate => 1500.0,
    }
}

/// Returns the temperature in kelvin and the moisture at the middle of the climate `species` lives in.
fn middle(species: &Species) -> (f32, f32) {
    let t = &species.tolerance;
    ((t.cold + t.heat) * 0.5, (t.dry + t.wet) * 0.5)
}

/// Returns the productivity of producer `species` of `life` on `world`, from 0 to 1: light times moisture
/// times how close its warmth is to the best of its solvent.
pub fn productivity(life: &Life, species: &Species, world: &World) -> f32 {
    let (warmth, moisture) = middle(species);
    let light = (world.flux.max(0.0) * 1.5 / (world.flux.max(0.0) + 0.5)).min(1.0);
    let off = (warmth - optimum(life.chemistry.solvent)) / 45.0;
    let fit = (1.0 - off * off).max(0.1);
    (light * moisture.clamp(0.05, 1.0) * fit).clamp(0.0, 1.0)
}

/// Returns the height in metres of architecture `arch` on `world` in a wind of `wind` metres per second, for
/// a group of `productivity` that lives in the `cold` or not: lower with gravity and with wind.
fn height(arch: u8, world: &World, wind: f32, productivity: f32, cold: bool) -> f32 {
    let pull = (EARTH_G / world.gravity.max(0.5)).sqrt().clamp(0.35, 2.2);
    let load = world.pressure.max(0.0) * (wind / GALE).powi(2);
    let chill = if cold { 0.7 } else { 1.0 };
    BASE[usize::from(arch.min(7))] * pull / (1.0 + 0.5 * load) * (0.6 + 0.6 * productivity) * chill
}

/// Returns true when an accent tissue of `species` is hard enough to show as thorns, or bites.
fn armed(species: &Species) -> bool {
    species
        .tissues
        .iter()
        .skip(1)
        .any(|t| t.hardness >= THORN || t.reaction != Reaction::Inert)
}

/// Returns the form of producer `species` of `life` on `world` in a wind of `wind` metres per second.
pub fn form(life: &Life, species: &Species, world: &World, wind: f32) -> Form {
    let (warmth, moisture) = middle(species);
    let best = optimum(life.chemistry.solvent);
    let (cold, hot, dry) = (warmth < best - COLD, warmth > best + HOT, moisture < DRY);
    let rich = productivity(life, species, world);
    let arch = super::architecture(life, species);
    let broad = (0.35 + 0.8 * moisture - 0.3 * (world.flux - 1.0)).clamp(0.15, 1.4);
    Form {
        height: height(arch, world, wind, rich, cold),
        layers: 1 + (rich * 3.0).round() as u8,
        leaf: if cold { broad.min(0.3) } else { broad },
        needles: cold,
        wax: dry,
        spines: moisture < 0.2 || (dry && hot),
        store: moisture < 0.25,
        thorns: armed(species),
        resin: species.tissues.iter().any(|t| t.kind == TissueKind::Resin),
        deciduous: !cold && !dry && matches!(arch, 0 | 3),
    }
}
