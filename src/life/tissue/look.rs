//! The look of tissues: base colours by kind, darker on wet worlds and paler on dry ones, warning
//! colours on toxic tissues, patterns by habitat, and strong value contrast between a main
//! tissue and its accents.

use super::super::chem::Setting;
use super::super::{Guild, Habitat, Pattern, Reaction, Role, Solvent, Tissue, TissueKind};
use crate::universe::hash::Rng;

/// Smallest gap in perceived value between the main tissue and an accent, 0 to 1.
const CONTRAST: f32 = 0.3;

/// Hide colour palettes a species draws from (ochre, terracotta, crimson, rust, amber, sand,
/// ivory, moss, emerald, teal, ocean blue, cobalt, slate, indigo, violet, plum, charcoal, copper).
const HIDES: [[f32; 3]; 18] = [
    [0.72, 0.50, 0.16],
    [0.68, 0.26, 0.14],
    [0.65, 0.11, 0.13],
    [0.52, 0.20, 0.09],
    [0.78, 0.40, 0.10],
    [0.74, 0.64, 0.42],
    [0.84, 0.80, 0.68],
    [0.24, 0.44, 0.16],
    [0.12, 0.50, 0.28],
    [0.10, 0.48, 0.48],
    [0.14, 0.34, 0.58],
    [0.16, 0.24, 0.68],
    [0.28, 0.34, 0.42],
    [0.24, 0.16, 0.54],
    [0.46, 0.18, 0.58],
    [0.48, 0.14, 0.34],
    [0.18, 0.18, 0.21],
    [0.62, 0.32, 0.18],
];

/// Shell and carapace palettes (coral, nacre, amber, slate, crimson, jade, bronze, cobalt).
const SHELLS: [[f32; 3]; 8] = [
    [0.72, 0.28, 0.18],
    [0.78, 0.74, 0.66],
    [0.70, 0.48, 0.14],
    [0.26, 0.32, 0.40],
    [0.54, 0.12, 0.18],
    [0.16, 0.46, 0.34],
    [0.50, 0.35, 0.18],
    [0.18, 0.28, 0.54],
];

/// Bone and horn palettes (ivory, aged bone, obsidian horn, warm ochre).
const BONES: [[f32; 3]; 4] = [
    [0.86, 0.82, 0.70],
    [0.72, 0.66, 0.52],
    [0.20, 0.18, 0.22],
    [0.78, 0.58, 0.34],
];

/// Resin and sap palettes (honey, emerald sap, crimson amber, violet pitch).
const RESINS: [[f32; 3]; 4] = [
    [0.78, 0.48, 0.10],
    [0.22, 0.56, 0.26],
    [0.64, 0.16, 0.14],
    [0.44, 0.20, 0.54],
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

/// Returns `colour` subtly tinted by seeded per-channel variation so siblings never share a flat wash.
fn shift(colour: [f32; 3], rng: &mut Rng) -> [f32; 3] {
    colour.map(|c| (c + (rng.unit() as f32 - 0.5) * 0.10).clamp(0.02, 0.98))
}

/// Returns the base colour of `kind` on the planet of `setting`, before wetness and warnings.
fn base(kind: TissueKind, setting: &Setting, rng: &mut Rng) -> [f32; 3] {
    let chemistry = &setting.chemistry;
    let pick = |list: &[[f32; 3]], r: &mut Rng| list[r.below(list.len() as u32) as usize];
    let c = match kind {
        TissueKind::Bone => pick(&BONES, rng),
        TissueKind::Hide => pick(&HIDES, rng),
        TissueKind::Shell => pick(&SHELLS, rng),
        TissueKind::Gel => chemistry.glow.map(|c| 0.35 + 0.55 * c),
        TissueKind::Crystal if chemistry.solvent == Solvent::Silicate => [0.9, 0.48, 0.18],
        TissueKind::Crystal => [0.58, 0.78, 0.94],
        TissueKind::Fibre => chemistry.pigment,
        TissueKind::Resin => pick(&RESINS, rng),
        TissueKind::Gland => [0.70, 0.22, 0.42],
    };
    shift(c, rng)
}

/// Returns `colour` darkened on a wet world and paled on a dry one, by the liquid share `ocean`.
fn humid(colour: [f32; 3], ocean: f32) -> [f32; 3] {
    let dark = 1.0 - 0.18 * ocean;
    let pale = 0.14 * (1.0 - ocean);
    colour.map(|c| {
        let c = c * dark;
        c + (1.0 - c) * pale
    })
}

/// Returns the pattern of a tissue of a species of `role`: bands on toxic ones, countershading on
/// swimmers and flyers, and seeded two-tone coats on most land species.
fn pattern(role: &Role, toxic: bool, rng: &mut Rng) -> Pattern {
    match (toxic, role.habitat, rng.below(20)) {
        (true, _, _) => Pattern::Bands,
        (_, _, _) if role.guild == Guild::Producer => Pattern::Plain,
        (_, Habitat::Water | Habitat::Air, _) => Pattern::Countershade,
        (_, _, 0..4) => Pattern::Plain,
        (_, _, 4..10) => Pattern::Stripes,
        (_, _, 10..16) => Pattern::Spots,
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
