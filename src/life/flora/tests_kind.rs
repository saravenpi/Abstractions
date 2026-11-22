//! Checks that a producer group shows several distinct species, the same on every visit, and that leaves
//! follow the light of their star.

use super::fixture::{MILD, life, of, producer, world};
use super::light::star;
use super::*;
use crate::life::chem::pigment;

/// Returns the luminance of linear colour `c`.
fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// A group that does well shows three species, a poor one two, each seeded the same on every visit, with
/// shares that sum to one and sizes and builds that differ.
#[test]
fn a_group_shows_distinct_species() {
    let group = producer(11, 2, 0, MILD);
    let (rich, poor) = (kinds(&group, 0.8), kinds(&group, 0.1));
    assert_eq!(
        (rich.len(), poor.len()),
        (usize::from(MOST), usize::from(MOST) - 1)
    );
    assert_eq!(rich, kinds(&group, 0.8));
    let total: f32 = rich.iter().map(|k| k.share).sum();
    assert!((total - 1.0).abs() < 1e-5, "shares sum to {total}");
    for (a, b) in rich.iter().zip(rich.iter().skip(1)) {
        let apart = (a.size - b.size).abs() + (a.spread - b.spread).abs() + (a.tint - b.tint).abs();
        assert!(apart > 0.05, "{a:?} and {b:?} look alike");
        assert!(a.share > b.share, "the commonest comes first");
    }
    assert_ne!(kinds(&producer(12, 2, 0, MILD), 0.8), rich);
}

/// The species of one group stand at different heights and wear different greens.
#[test]
fn species_of_a_group_differ_in_size_and_tint() {
    let planet = life(
        Rung::Skeletal,
        pigment(5800.0),
        vec![producer(3, 0, 0, MILD)],
        vec![],
    );
    let s = &planet.species[0];
    let looks: Vec<Dials> = kinds(s, 0.8)
        .into_iter()
        .map(|k| of_kind(&planet, s, (&world(9.81, 5800.0), 3.0), Stage::Green, k))
        .collect();
    for (a, b) in looks.iter().zip(looks.iter().skip(1)) {
        assert_ne!(a.form.height, b.form.height);
        assert_ne!(a.paint.leaves, b.paint.leaves);
    }
}

/// The star read back from a pigment is the star that made it, within a hundred kelvin.
#[test]
fn the_star_is_read_back_from_its_pigment() {
    for kelvin in (25..=110).map(|k| k as f32 * 100.0) {
        let read = star(pigment(kelvin));
        assert!(
            (read - kelvin).abs() <= 100.0,
            "{kelvin} K read as {read} K"
        );
    }
}

/// Leaves are nearly black under a red dwarf, green under a yellow star, and paler and redder than their
/// pigment under a hot star.
#[test]
fn leaves_follow_the_temperature_of_the_star() {
    let leaves = |kelvin: f32| {
        let planet = life(
            Rung::Skeletal,
            pigment(kelvin),
            vec![producer(1, 0, 0, MILD)],
            vec![],
        );
        of(&planet, 0, Stage::Green).paint.leaves.expect("in leaf")
    };
    let (red, sun, blue) = (leaves(2900.0), leaves(5800.0), leaves(9500.0));
    assert!(luma(red) < 0.03, "{red:?} under a red dwarf");
    assert!(luma(red) < 0.3 * luma(sun), "{red:?} against {sun:?}");
    assert!(luma(blue) > luma(sun), "{blue:?} against {sun:?}");
    let raw = pigment(9500.0);
    assert!(
        blue[0] / blue[2] > raw[0] / raw[2],
        "{blue:?} against its pigment {raw:?}"
    );
}

/// Alien solvents and skeletal structures shift stem hues away from earthlike dark bark.
#[test]
fn alien_biochemistry_varies_stem_colours() {
    use crate::life::{Solvent, Structure};

    let home = life(
        Rung::Skeletal,
        pigment(5800.0),
        vec![producer(1, 0, 0, MILD)],
        vec![],
    );
    let mut ammonia = home.clone();
    ammonia.chemistry.solvent = Solvent::Ammonia;
    ammonia.chemistry.structure = Structure::Silica;
    let mut metal = home.clone();
    metal.chemistry.solvent = Solvent::Silicate;
    metal.chemistry.structure = Structure::Metal;
    let a = of(&home, 0, Stage::Green).paint.stem;
    let b = of(&ammonia, 0, Stage::Green).paint.stem;
    let c = of(&metal, 0, Stage::Green).paint.stem;
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_ne!(b, c);
}
