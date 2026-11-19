//! Kodya, the home planet: home chemistry, a grazer that lives in herds on the ground, and a hunter.

use super::super::chem::{self, Setting};
use super::super::{Chemistry, Energy, Guild, Habitat, Plan, Solvent, Species, Structure, World};
use super::fill;
use crate::universe::hash::child;

/// Fewest members of the largest herd of the home grazer.
const HERD: u8 = 6;

/// Returns the chemistry of home: water, starlight and carbon, with the pigment of its star.
pub(super) fn chemistry(world: &World, seed: u64) -> Chemistry {
    Chemistry {
        solvent: Solvent::Water,
        energy: Energy::Light,
        structure: Structure::Carbon,
        ..chem::chemistry(world, seed)
    }
}

/// Turns `species` into a skeletal ground animal of `guild` and `mass` kilograms, keeping its place in the web.
fn recast(species: &mut Species, guild: Guild, mass: f32, setting: &Setting) {
    let role = super::super::Role {
        guild,
        habitat: Habitat::Ground,
        ..species.role
    };
    let seed = child(species.id.planet, 0x484F_0000 + u64::from(species.id.index));
    let fresh = fill::reform(seed, (Plan::Skeletal, 0), role, mass, setting);
    *species = Species {
        id: species.id,
        prey: std::mem::take(&mut species.prey),
        ..fresh
    };
}

/// Makes sure the home web has a skeletal ground grazer that lives in herds of at least six, and a hunter.
///
/// The grazer is the first consumer that eats only producers, the hunter the consumer highest in
/// the web. Each is recast only when the web did not already draw one.
pub(super) fn settle(species: &mut [Species], producers: usize, setting: &Setting) {
    let herd = |s: &Species| {
        s.role.guild == Guild::Grazer
            && s.role.habitat == Habitat::Ground
            && s.plan == Plan::Skeletal
    };
    let grazer = species.iter().position(herd).or_else(|| {
        let plant_eater = |s: &&Species| s.prey.iter().all(|&p| usize::from(p) < producers);
        species
            .iter()
            .skip(producers)
            .position(|s| plant_eater(&s))
            .map(|i| i + producers)
    });
    let Some(grazer) = grazer.or((producers < species.len()).then_some(producers)) else {
        return;
    };
    if !herd(&species[grazer]) {
        recast(&mut species[grazer], Guild::Grazer, 180.0, setting);
    }
    let history = &mut species[grazer].history;
    history.group_max = history.group_max.max(HERD);
    history.group_min = history.group_min.max(3);
    if species.iter().all(|s| s.role.guild != Guild::Hunter) {
        let top = (producers..species.len())
            .filter(|&i| i != grazer)
            .max_by(|&a, &b| species[a].role.level.total_cmp(&species[b].role.level));
        if let Some(top) = top {
            recast(&mut species[top], Guild::Hunter, 60.0, setting);
        }
    }
}
