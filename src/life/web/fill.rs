//! Filling the species of a drawn web: roles, plans, masses, tissues, life histories and bodies.

use super::super::body::{self, Body, Locomotion};
use super::super::chem::Setting;
use super::super::{
    Activity, Guild, Habitat, Plan, Role, Rung, Solvent, Species, SpeciesId, Tolerance,
};
use super::super::{guild, history, tissue};
use super::mass::mass;
use super::niche::Web;
use crate::universe::hash::{Rng, child};

/// Growth architecture index of kelp, the one producer that lives in the liquid.
const KELP: u8 = 6;

/// Returns the body plan of a consumer in `habitat` on the planet of `setting`, weighted by the rung.
///
/// Soft life is slimes and jellies, shelled life adds segments, swarms and crystals, and skeletons
/// come with the skeletal rung, where they are the most common plan. Molten silicate life is mostly crystal.
fn plan(habitat: Habitat, setting: &Setting, rng: &mut Rng) -> Plan {
    let shelled = u32::from(setting.rung >= Rung::Shelled);
    let crystal = match setting.chemistry.solvent {
        Solvent::Silicate => 6,
        _ => 1,
    };
    let weights = [
        (
            Plan::Skeletal,
            6 * u32::from(setting.rung >= Rung::Skeletal),
        ),
        (Plan::Amorphous, 1 + u32::from(shelled == 0)),
        (Plan::Radial, 1 + u32::from(habitat == Habitat::Water)),
        (Plan::Segmented, 2 * shelled),
        (Plan::Colonial, shelled),
        (Plan::Crystalline, crystal * shelled),
    ];
    let total: u32 = weights.iter().map(|w| w.1).sum();
    let mut roll = rng.below(total);
    for (plan, weight) in weights {
        if roll < weight {
            return plan;
        }
        roll -= weight;
    }
    Plan::Amorphous
}

/// Returns the climate a species of `mass` in `habitat` stands on `setting`: wider for larger species.
fn tolerance(mass: f32, habitat: Habitat, setting: &Setting) -> Tolerance {
    let t = setting.world.temperature;
    let span = 15.0 + 5.0 * mass.max(1e-3).log10().max(0.0);
    let wet = setting.world.ocean;
    Tolerance {
        cold: t - span - 10.0,
        heat: t + span,
        dry: if habitat == Habitat::Water {
            0.8
        } else {
            (wet - 0.5).max(0.0)
        },
        wet: 1.0,
    }
}

/// Fills a species of `plan`, `archetype`, `role` and `mass` from `seed`: its history, tissues,
/// glow, tolerance and body. Name, epithet and signature stay empty for the names step.
fn reform(seed: u64, form: (Plan, u8), role: Role, mass: f32, setting: &Setting) -> Species {
    let mut rng = Rng::new(child(seed, 2));
    let history = history::of(seed, &role, mass, setting.rung, &setting.world);
    let mut tissues = tissue::of(seed, form.0, &role, setting);
    let glow = guild::glow(&role, &tissues, setting, &mut rng);
    if let (Some(glow), Some(last)) = (glow, tissues.last_mut()) {
        last.glow = glow.strength;
    }
    let body = match role.guild {
        Guild::Producer => empty(),
        _ => body::build(seed, form, &role, &history, &setting.world),
    };
    Species {
        id: SpeciesId {
            planet: 0,
            index: 0,
        },
        name: String::new(),
        epithet: String::new(),
        plan: form.0,
        archetype: form.1,
        role,
        tolerance: tolerance(mass, role.habitat, setting),
        history,
        signature: tissues[0].kind,
        tissues,
        glow,
        body,
        prey: Vec::new(),
    }
}

/// Returns the empty body every producer has.
fn empty() -> Body {
    Body {
        parts: Vec::new(),
        legs: Vec::new(),
        spine: None,
        neck: None,
        tail: None,
        tentacles: Vec::new(),
        wings: None,
        locomotion: Locomotion::Walk,
        length: 0.0,
        height: 0.0,
        features: Vec::new(),
    }
}

/// Returns the form, role and mass of producer number `index` of `web`, of growth `architecture`.
fn producer(web: &Web, index: usize, architecture: u8) -> ((Plan, u8), Role, f32) {
    let niche = web.niche[index];
    let role = Role {
        guild: Guild::Producer,
        habitat: if architecture == KELP {
            Habitat::Water
        } else {
            Habitat::Ground
        },
        activity: Activity::Day,
        niche,
        range: 0.0,
        centre: 0.0,
        level: 1.0,
    };
    (
        (Plan::Colonial, architecture),
        role,
        10f32.powf(-1.0 + 4.0 * niche),
    )
}

/// Returns the form, role and mass of consumer number `index` of `web` on the planet of `setting`.
fn consumer(seed: u64, web: &Web, index: usize, setting: &Setting) -> ((Plan, u8), Role, f32) {
    let mut rng = Rng::new(child(seed, 1));
    let (niche, level) = (web.niche[index], web.level[index]);
    let guild = guild::of(level, niche, setting, &mut rng);
    let habitat = guild::habitat(guild, setting, &mut rng);
    let role = Role {
        guild,
        habitat,
        activity: guild::activity(guild, habitat, setting, &mut rng),
        niche,
        range: web.range[index],
        centre: web.centre[index],
        level,
    };
    let form = (plan(habitat, setting, &mut rng), rng.below(4) as u8);
    let (form, mass) = mass(form, (habitat, niche, level), setting, &mut rng);
    (form, role, mass)
}

/// Returns species number `index` of `web` on the planet of seed `planet`, with `architecture` the
/// growth architecture when it is a producer.
pub(super) fn species(
    planet: u64,
    index: usize,
    web: &Web,
    architecture: u8,
    setting: &Setting,
) -> Species {
    let seed = child(planet, 0x5350_0000 + index as u64);
    let (form, role, mass) = match index < web.producers {
        true => producer(web, index, architecture),
        false => consumer(seed, web, index, setting),
    };
    let mut species = reform(seed, form, role, mass, setting);
    species.id = SpeciesId {
        planet,
        index: index as u16,
    };
    species.prey = web.prey[index].clone();
    species
}
