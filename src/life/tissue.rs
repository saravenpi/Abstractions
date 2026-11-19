//! Tissues: the planet sets the baseline of each kind, habitat and niche shift it per species.
//!
//! Silica life is hard, brittle and heat proof, metal shells are dense and conduct, and the life
//! of the cold solvents insulates. A species has at most three tissues: a main one and two accents.

mod look;

use super::chem::Setting;
use super::{
    Chemistry, Guild, Habitat, Plan, Reaction, Role, Rung, Solvent, Structure, Tissue, TissueKind,
};
use crate::universe::hash::{Rng, child};

#[cfg(test)]
pub(super) mod tests;

/// Child of the species seed the tissue draws come from.
const STREAM: u64 = 0x54_49_53;

/// Returns the physics of `kind` under `chemistry`: hardness, toughness, density, elasticity, heat and
/// conductivity, from the home values shifted by the structure and the solvent.
fn physics(kind: TissueKind, chemistry: &Chemistry) -> [f32; 6] {
    let [h, t, d, e, heat, c] = match kind {
        TissueKind::Bone => [5.0, 0.6, 1900.0, 0.2, 400.0, 0.2],
        TissueKind::Hide => [1.5, 0.7, 1100.0, 0.6, 350.0, 0.1],
        TissueKind::Shell => [4.0, 0.4, 2300.0, 0.1, 450.0, 0.2],
        TissueKind::Gel => [0.2, 0.2, 1050.0, 0.9, 330.0, 0.5],
        TissueKind::Crystal => [7.0, 0.15, 2600.0, 0.05, 1500.0, 0.3],
        TissueKind::Fibre => [2.0, 0.8, 900.0, 0.5, 380.0, 0.1],
        TissueKind::Resin => [2.5, 0.3, 1200.0, 0.3, 420.0, 0.05],
        TissueKind::Gland => [0.5, 0.3, 1050.0, 0.7, 330.0, 0.3],
    };
    let [h, t, d, heat, c] = match chemistry.structure {
        Structure::Carbon => [h, t, d, heat, c],
        Structure::Silica => [h + 2.0, t * 0.6, d, heat + 800.0, c],
        Structure::Metal => [h + 1.5, t, d * 1.6, heat, c + 0.5],
    };
    match chemistry.solvent {
        Solvent::Ammonia | Solvent::Hydrocarbon => [h, t * 1.2, d, e, heat, c * 0.4],
        Solvent::Silicate => [h, t, d, e, heat.max(1800.0), c],
        Solvent::Water => [h, t, d, e, heat, c],
    }
}

/// Returns the baseline of `kind` on the planet of `setting`: shifted by its structure and solvent, and
/// always standing at least 60 K over the mean surface temperature.
pub fn baseline(kind: TissueKind, setting: &Setting) -> Tissue {
    let [hardness, toughness, density, elasticity, heat, conductivity] =
        physics(kind, &setting.chemistry);
    Tissue {
        kind,
        hardness: hardness.min(10.0),
        toughness: toughness.min(1.0),
        density,
        elasticity,
        heat: heat.max(setting.world.temperature + 60.0),
        conductivity: conductivity.min(1.0),
        reaction: Reaction::Inert,
        colour: [0.5; 3],
        pattern: super::Pattern::Plain,
        gloss: 0.0,
        translucency: 0.0,
        glow: 0.0,
    }
}

/// Shifts `tissue` for a species of `role` at `rung`: light in the air, supple in water, and weapons
/// and armour hardening together as the rung climbs.
fn adapt(tissue: &mut Tissue, role: &Role, rung: Rung) {
    let race = 0.4 * rung as u8 as f32;
    match role.habitat {
        Habitat::Air => tissue.density *= 0.7,
        Habitat::Water => tissue.elasticity = (tissue.elasticity + 0.15).min(1.0),
        Habitat::Ground | Habitat::Shore => {}
    }
    let armour = matches!(
        tissue.kind,
        TissueKind::Bone | TissueKind::Shell | TissueKind::Crystal
    );
    if armour && role.guild != Guild::Producer {
        tissue.hardness = (tissue.hardness + race * role.level.min(4.0) / 4.0).min(10.0);
    }
}

/// Returns the tissue kinds of a species of `plan` and `guild`: the main kind and its accents.
fn kinds(plan: Plan, guild: Guild, setting: &Setting, rng: &mut Rng) -> Vec<TissueKind> {
    use TissueKind::*;
    let silica = setting.chemistry.structure == Structure::Silica;
    let list: &[TissueKind] = match (guild, plan) {
        (Guild::Producer, _) if silica => &[Crystal, Resin, Gland],
        (Guild::Producer, _) => &[Fibre, Resin, Gland],
        (_, Plan::Skeletal) if setting.rung == Rung::Old => &[Hide, Bone, Shell],
        (_, Plan::Skeletal) => &[Hide, Bone, Gland],
        (_, Plan::Amorphous) => &[Gel, Gland],
        (_, Plan::Radial) => &[Gel, Fibre, Gland],
        (_, Plan::Segmented) => &[Shell, Hide, Gland],
        (_, Plan::Colonial) => &[Shell, Gland],
        (_, Plan::Crystalline) => &[Crystal, Shell, Resin],
    };
    let keep = (2 + rng.below(2) as usize).min(list.len());
    list[..keep].to_vec()
}

/// Returns what a tissue of `kind` does to what touches it: glands turn toxic more often as the rung climbs.
fn reaction(kind: TissueKind, setting: &Setting, rng: &mut Rng) -> Reaction {
    let odds = match setting.rung {
        Rung::Mats | Rung::Soft => 0.1,
        Rung::Shelled | Rung::Skeletal => 0.4,
        Rung::Old => 0.6,
    };
    let toxic = kind == TissueKind::Gland && rng.unit() < odds;
    match (toxic, setting.chemistry.structure, rng.below(2)) {
        (false, _, _) => Reaction::Inert,
        (true, Structure::Metal, _) => Reaction::Charge,
        (true, _, 0) => Reaction::Venom,
        (true, _, _) => Reaction::Acid,
    }
}

/// Returns the tissues of a species of `plan` and `role` on the planet of `setting`, rolled from `seed`:
/// the main tissue first, then one or two accents in strong value contrast with it.
pub fn of(seed: u64, plan: Plan, role: &Role, setting: &Setting) -> Vec<Tissue> {
    let mut rng = Rng::new(child(seed, STREAM));
    let kinds = kinds(plan, role.guild, setting, &mut rng);
    let mut tissues: Vec<Tissue> = kinds
        .iter()
        .map(|&kind| {
            let mut tissue = baseline(kind, setting);
            tissue.reaction = reaction(kind, setting, &mut rng);
            adapt(&mut tissue, role, setting.rung);
            look::dress(&mut tissue, role, setting, &mut rng);
            tissue
        })
        .collect();
    look::contrast(&mut tissues);
    tissues
}

/// Returns the traits of a tissue the signature compares, each about 0 to 1: hardness, toughness,
/// density, heat, value and glow.
pub fn traits(tissue: &Tissue) -> [f32; 6] {
    [
        tissue.hardness / 10.0,
        tissue.toughness,
        tissue.density / 3000.0,
        tissue.heat / 3000.0,
        look::value(tissue.colour),
        tissue.glow,
    ]
}

/// Returns the kind of the tissue among `tissues` whose traits lie farthest from the planet `average`.
pub fn signature(tissues: &[Tissue], average: &[f32; 6]) -> TissueKind {
    let distance = |t: &Tissue| -> f32 {
        traits(t)
            .iter()
            .zip(average)
            .map(|(a, b)| (a - b) * (a - b))
            .sum()
    };
    tissues
        .iter()
        .max_by(|a, b| distance(a).total_cmp(&distance(b)))
        .map_or(TissueKind::Hide, |t| t.kind)
}
