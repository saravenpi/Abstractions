//! The food web of a planet, after the niche model, and the assembly of its whole life.

mod fill;
mod home;
mod niche;
mod plants;

use super::chem::{self, Setting};
use super::{Life, Rung, Species, World, names, odds, relate, tissue};
use crate::universe::hash::{Rng, child};
use crate::universe::{HOME_PLANET_NAME, PlanetSpec, StarSpec};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cost;

/// Child of the planet seed the web draws come from.
const STREAM: u64 = 0x57_45_42;

/// Returns the productivity of the planet of `setting`, from 0 to 1: light times moisture times how
/// well its liquid fits the temperature. Producers living on vents or heat get half the light.
fn productivity(setting: &Setting) -> f32 {
    let world = &setting.world;
    let light = match setting.chemistry.energy {
        super::Energy::Light => (1.25 * world.flux / (world.flux + 0.25)).min(1.0),
        _ => 0.5,
    };
    let moisture = 0.3 + 0.7 * (world.ocean / 0.6).min(1.0);
    let fit = chem::solvent(world).map_or(0.3, |(_, fit)| fit.max(0.3));
    light * moisture * fit
}

/// Returns the shape of the web of the planet of `setting`.
///
/// Mats hold one to three producers and nothing else. Above them the species count runs from 12 to
/// 40 with productivity and the rung, a fifth to a quarter of them producers, at a connectance of
/// 0.10 to 0.15, and soft life keeps its chains short.
fn shape(setting: &Setting, rng: &mut Rng) -> niche::Shape {
    let depth = [0.0, 0.25, 0.55, 0.8, 1.0][setting.rung as usize];
    let jitter = rng.range(-2.0, 2.0) as f32;
    let count = 12.0 + 28.0 * productivity(setting) * depth + jitter;
    let species = count.round().clamp(12.0, 40.0) as usize;
    let mats = 1 + rng.below(3) as usize;
    let producers = ((species as f32 * 0.22).round() as usize).clamp(2, 8);
    let connectance = rng.range(0.1, 0.15) as f32;
    let reach = (setting.rung == Rung::Soft).then_some(0.15);
    match setting.rung {
        Rung::Mats => niche::Shape {
            species: mats,
            producers: mats,
            connectance,
            reach,
        },
        _ => niche::Shape {
            species,
            producers,
            connectance,
            reach,
        },
    }
}

/// Sets the signature of every species of a planet: its tissue farthest from the planet's average.
fn sign(species: &mut [Species]) {
    let all: Vec<[f32; 6]> = species
        .iter()
        .flat_map(|s| s.tissues.iter().map(tissue::traits))
        .collect();
    let mut average = [0.0; 6];
    for traits in &all {
        (0..6).for_each(|i| average[i] += traits[i] / all.len().max(1) as f32);
    }
    for s in species.iter_mut() {
        s.signature = tissue::signature(&s.tissues, &average);
    }
}

/// Returns every species of the planet of seed `seed` and `setting`, producers first, and how many
/// producers there are.
fn species(seed: u64, setting: &Setting) -> (Vec<Species>, usize) {
    let mut rng = Rng::new(child(seed, STREAM));
    let web = niche::draw(shape(setting, &mut rng), &mut rng);
    let architectures = plants::architectures(web.producers, setting, &mut rng);
    let species = (0..web.niche.len())
        .map(|i| {
            let architecture = architectures.get(i).copied().unwrap_or(2);
            fill::species(seed, i, &web, architecture, setting)
        })
        .collect();
    (species, web.producers)
}

/// Returns the whole life of `planet` around `star`, or nothing on a dead world.
///
/// Kodya, the home planet, always lives at the skeletal rung with home chemistry, a herd grazer and
/// a hunter. Species come producers first, `species[i].id.index == i`.
pub fn assemble(planet: &PlanetSpec, star: &StarSpec) -> Option<Life> {
    let world = World::of(planet, star);
    let is_home = planet.name == HOME_PLANET_NAME && planet.id.moon == 0;
    let (rung, chemistry) = match is_home {
        true => (Rung::Skeletal, home::chemistry(&world, planet.seed)),
        false => (
            odds::rung(&world, planet.seed)?,
            chem::chemistry(&world, planet.seed),
        ),
    };
    let setting = Setting {
        world,
        chemistry,
        rung,
    };
    let (mut species, producers) = species(planet.seed, &setting);
    if is_home {
        home::settle(&mut species, producers, &setting);
    }
    sign(&mut species);
    names::christen(&mut species, names::region(star));
    let (relations, keystone) = relate::of(&mut species);
    Some(Life {
        rung,
        chemistry,
        species,
        relations,
        keystone,
    })
}
