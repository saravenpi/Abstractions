//! Tests of tissue baselines, looks and signatures.

use super::look::value;
use super::*;
use crate::life::{Activity, Chemistry, Energy, World, chem};

/// Returns a setting at `rung` with home air and `structure`, `solvent` and liquid share `ocean`.
pub(in crate::life) fn setting(
    rung: Rung,
    structure: Structure,
    solvent: Solvent,
    ocean: f32,
) -> Setting {
    let world = World {
        gravity: 22.0,
        pressure: 1.0,
        temperature: 288.0,
        flux: 1.0,
        radius: 6000.0,
        ocean,
        star: 5780.0,
        tilt: 0.0,
        liquids: [
            ocean.max(f32::MIN_POSITIVE),
            0.0375,
            0.0375,
            ocean.max(f32::MIN_POSITIVE),
        ],
    };
    let chemistry = Chemistry {
        solvent,
        energy: Energy::Light,
        structure,
        pigment: chem::pigment(5780.0),
        glow: [0.2, 0.9, 0.7],
    };
    Setting {
        world,
        chemistry,
        rung,
    }
}

/// Returns the role of a ground consumer of `guild` at trophic `level`.
pub(in crate::life) fn role(guild: Guild, level: f32) -> Role {
    Role {
        guild,
        habitat: Habitat::Ground,
        activity: Activity::Day,
        niche: 0.5,
        range: 0.1,
        centre: 0.4,
        level,
    }
}

/// Silica bones are harder and stand more heat than carbon ones, and cold solvent life insulates.
#[test]
fn chemistry_sets_the_baseline() {
    let carbon = baseline(
        TissueKind::Bone,
        &setting(Rung::Skeletal, Structure::Carbon, Solvent::Water, 0.5),
    );
    let silica = baseline(
        TissueKind::Bone,
        &setting(Rung::Skeletal, Structure::Silica, Solvent::Water, 0.5),
    );
    let cold = baseline(
        TissueKind::Hide,
        &setting(Rung::Skeletal, Structure::Carbon, Solvent::Ammonia, 0.5),
    );
    let warm = baseline(
        TissueKind::Hide,
        &setting(Rung::Skeletal, Structure::Carbon, Solvent::Water, 0.5),
    );
    assert!(
        silica.hardness > carbon.hardness
            && silica.heat > carbon.heat
            && silica.toughness < carbon.toughness
    );
    assert!(cold.conductivity < warm.conductivity);
}

/// A species has one to three tissues, accents stand apart in value, and toxic tissues show warning colours.
#[test]
fn tissues_read_clearly() {
    let at = setting(Rung::Old, Structure::Carbon, Solvent::Water, 0.5);
    for seed in 0..300 {
        let tissues = of(seed, Plan::Skeletal, &role(Guild::Hunter, 3.0), &at);
        assert!((1..=3).contains(&tissues.len()));
        assert_eq!(
            tissues,
            of(seed, Plan::Skeletal, &role(Guild::Hunter, 3.0), &at)
        );
        let main = value(tissues[0].colour);
        assert!(
            tissues[1..]
                .iter()
                .all(|t| (value(t.colour) - main).abs() > 0.29),
            "seed {seed}"
        );
        for t in tissues.iter().filter(|t| t.reaction != Reaction::Inert) {
            assert!(
                t.colour.iter().copied().fold(0.0, f32::max) > 0.75
                    && t.pattern == crate::life::Pattern::Bands
            );
        }
    }
}

/// The same main tissue is darker on a wet world than on a dry one.
#[test]
fn wet_worlds_are_darker() {
    let main = |ocean| {
        of(
            4,
            Plan::Segmented,
            &role(Guild::Grazer, 2.0),
            &setting(Rung::Shelled, Structure::Carbon, Solvent::Water, ocean),
        )[0]
    };
    assert!(value(main(0.9).colour) < value(main(0.0).colour));
}

/// Different seeded species on the same planet exhibit distinct coat colours across the palette.
#[test]
fn species_colours_vary_widely_across_seeds() {
    use std::collections::HashSet;

    let at = setting(Rung::Skeletal, Structure::Carbon, Solvent::Water, 0.4);
    let swatches: HashSet<[u8; 3]> = (0..64)
        .map(|s| of(s, Plan::Skeletal, &role(Guild::Grazer, 2.0), &at)[0].colour)
        .map(|c| c.map(|v| (v * 8.0).round() as u8))
        .collect();
    assert!(
        swatches.len() >= 12,
        "only {} unique swatches",
        swatches.len()
    );
}

/// The signature is the tissue farthest from the planet average.
#[test]
fn signature_is_the_outlier() {
    let at = setting(Rung::Skeletal, Structure::Carbon, Solvent::Water, 0.5);
    let tissues = [
        baseline(TissueKind::Hide, &at),
        baseline(TissueKind::Crystal, &at),
    ];
    let average = traits(&baseline(TissueKind::Hide, &at));
    assert_eq!(signature(&tissues, &average), TissueKind::Crystal);
}
