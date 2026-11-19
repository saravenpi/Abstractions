//! The look of tissues: base colours by kind, darker on wet worlds and paler on dry ones, warning
//! colours on toxic tissues, patterns by habitat, and strong value contrast between a main
//! tissue and its accents.

use super::super::chem::Setting;
use super::super::{Guild, Habitat, Pattern, Reaction, Role, Solvent, Tissue, TissueKind};
use crate::universe::hash::Rng;

/// Smallest gap in perceived value between the main tissue and an accent, 0 to 1.
const CONTRAST: f32 = 0.3;

/// Earthy hide colours a species draws from.
const HIDES: [[f32; 3]; 6] = [
    [0.35, 0.22, 0.12],
    [0.5, 0.42, 0.3],
    [0.25, 0.25, 0.27],
    [0.6, 0.45, 0.2],
    [0.2, 0.28, 0.22],
    [0.45, 0.3, 0.35],
];

/// Bright colours toxic tissues show: yellow, orange, red and magenta.
const WARNINGS: [[f32; 3]; 4] = [
    [0.95, 0.8, 0.05],
    [0.95, 0.4, 0.02],
    [0.85, 0.05, 0.05],
    [0.8, 0.1, 0.6],
];

/// Returns the perceived value of a linear RGB colour, 0 for black to 1 for white.
pub(super) fn value(colour: [f32; 3]) -> f32 {
    (0.2126 * colour[0] + 0.7152 * colour[1] + 0.0722 * colour[2])
        .max(0.0)
        .powf(1.0 / 2.2)
}

/// Returns the base colour of `kind` on the planet of `setting`, before wetness and warnings.
fn base(kind: TissueKind, setting: &Setting, rng: &mut Rng) -> [f32; 3] {
    let chemistry = &setting.chemistry;
    match kind {
        TissueKind::Bone => [0.8, 0.75, 0.6],
        TissueKind::Hide => HIDES[rng.below(HIDES.len() as u32) as usize],
        TissueKind::Shell => [0.5, 0.35, 0.25],
        TissueKind::Gel => chemistry.glow.map(|c| 0.4 + 0.5 * c),
        TissueKind::Crystal if chemistry.solvent == Solvent::Silicate => [0.9, 0.5, 0.2],
        TissueKind::Crystal => [0.7, 0.8, 0.9],
        TissueKind::Fibre => chemistry.pigment,
        TissueKind::Resin => [0.7, 0.4, 0.1],
        TissueKind::Gland => [0.6, 0.2, 0.3],
    }
}

/// Returns `colour` darkened on a wet world and paled on a dry one, by the liquid share `ocean`.
fn humid(colour: [f32; 3], ocean: f32) -> [f32; 3] {
    let dark = 1.0 - 0.35 * ocean;
    let pale = 0.25 * (1.0 - ocean);
    colour.map(|c| {
        let c = c * dark;
        c + (1.0 - c) * pale
    })
}

/// Returns the pattern of a tissue of a species of `role`: bands on toxic ones, countershading on
/// swimmers and flyers, and a seeded coat on the rest.
fn pattern(role: &Role, toxic: bool, rng: &mut Rng) -> Pattern {
    match (toxic, role.habitat, rng.below(20)) {
        (true, _, _) => Pattern::Bands,
        (_, _, _) if role.guild == Guild::Producer => Pattern::Plain,
        (_, Habitat::Water | Habitat::Air, _) => Pattern::Countershade,
        (_, _, 0..8) => Pattern::Plain,
        (_, _, 8..12) => Pattern::Stripes,
        (_, _, 12..17) => Pattern::Spots,
        _ => Pattern::Bands,
    }
}

/// Sets the colour, pattern, gloss and translucency of `tissue` for a species of `role` on the planet of `setting`.
pub(super) fn dress(tissue: &mut Tissue, role: &Role, setting: &Setting, rng: &mut Rng) {
    let toxic = tissue.reaction != Reaction::Inert;
    tissue.colour = match toxic {
        true => WARNINGS[rng.below(WARNINGS.len() as u32) as usize],
        false => humid(base(tissue.kind, setting, rng), setting.world.ocean),
    };
    tissue.pattern = pattern(role, toxic, rng);
    (tissue.gloss, tissue.translucency) = match tissue.kind {
        TissueKind::Gel => (0.8, 0.6),
        TissueKind::Crystal => (0.9, 0.5),
        TissueKind::Shell => (0.6, 0.0),
        TissueKind::Resin => (0.7, 0.3),
        _ => (0.2, 0.0),
    };
}

/// Returns `colour` moved to the perceived value `target`, keeping its hue: scaled down to darken
/// and mixed with white to lighten.
fn to_value(colour: [f32; 3], target: f32) -> [f32; 3] {
    let luma = |c: [f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let (now, goal) = (luma(colour).max(1e-4), target.clamp(0.0, 1.0).powf(2.2));
    match goal < now {
        true => colour.map(|c| c * goal / now),
        false => colour.map(|c| c + (1.0 - c) * (goal - now) / (1.0 - now).max(1e-4)),
    }
}

/// Moves every accent of `tissues` at least [`CONTRAST`] away in value from the main tissue, the
/// first one: darker under a pale main tissue, paler under a dark one.
pub(super) fn contrast(tissues: &mut [Tissue]) {
    let Some((main, accents)) = tissues.split_first_mut() else {
        return;
    };
    let base = value(main.colour);
    let away = if base > 0.5 { -1.0 } else { 1.0 };
    for accent in accents {
        let gap = (value(accent.colour) - base) * away;
        if gap < CONTRAST {
            accent.colour = to_value(accent.colour, base + away * (CONTRAST + 0.1));
        }
    }
}
