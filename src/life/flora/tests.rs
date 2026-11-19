//! Checks that plants follow their planet: gravity, wind, cold, drought, light, pollinators and seasons.

use super::fixture::{GREEN, MILD, garden, life, producer, world};
use super::*;
use crate::life::{Relation, Tolerance};

/// Pigment of a planet under a red dwarf of 3000 K, red to black as the roadmap gives it.
const RED_DWARF: [f32; 3] = [0.12, 0.015, 0.01];

/// Returns the dials of producer `i` of `life` on a home-like world at `stage`.
fn of(life: &Life, i: usize, stage: Stage) -> Dials {
    dials(life, &life.species[i], &world(9.81, 5800.0), 3.0, stage)
}

/// Returns the luma of a linear colour.
fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Under a 3000 K star the leaves are redder and darker than under a 5800 K one.
#[test]
fn leaves_under_a_red_dwarf_are_redder_and_darker() {
    let leaves = |pigment| {
        let life = life(
            Rung::Skeletal,
            pigment,
            vec![producer(1, 0, 0, MILD)],
            vec![],
        );
        of(&life, 0, Stage::Green).paint.leaves.expect("in leaf")
    };
    let (red, sun) = (leaves(RED_DWARF), leaves(GREEN));
    assert!(red[0] / red[1] > sun[0] / sun[1], "{red:?} against {sun:?}");
    assert!(luma(red) < luma(sun), "{red:?} against {sun:?}");
}

/// A tree on a 2 g world is lower than on a 0.5 g one, and lower again in a strong wind.
#[test]
fn gravity_and_wind_lower_plants() {
    let life = garden(1, &[0], vec![]);
    let tall = |g: f32, wind: f32| {
        dials(
            &life,
            &life.species[0],
            &world(g, 5800.0),
            wind,
            Stage::Ever,
        )
    };
    let (heavy, light) = (
        tall(2.0 * 9.81, 3.0).form.height,
        tall(0.5 * 9.81, 3.0).form.height,
    );
    assert!(heavy < light, "{heavy} m at 2 g, {light} m at 0.5 g");
    assert!(tall(9.81, 25.0).form.height < tall(9.81, 2.0).form.height);
}

/// No group flowers without a pollinator, and only the pollinated one flowers in spring.
#[test]
fn no_flowers_without_a_pollinator() {
    let bare = garden(1, &[0, 1, 3, 4], vec![]);
    let stages = [
        Stage::Ever,
        Stage::Bloom,
        Stage::Green,
        Stage::Fruit,
        Stage::Dormant,
    ];
    for (i, stage) in (0..4).flat_map(|i| stages.map(|s| (i, s))) {
        assert!(of(&bare, i, stage).paint.flowers.is_none());
    }
    let visited = garden(
        1,
        &[0, 1],
        vec![Relation::Pollinates {
            animal: 9,
            plant: 1,
        }],
    );
    assert!(of(&visited, 0, Stage::Bloom).paint.flowers.is_none());
    assert!(of(&visited, 1, Stage::Bloom).paint.flowers.is_some());
    assert!(of(&visited, 1, Stage::Dormant).paint.flowers.is_none());
}

/// Fruit hangs only where a seed carrier lives, in autumn or all year in the tropics.
#[test]
fn fruit_only_with_a_seed_carrier() {
    let sown = garden(
        1,
        &[0, 0],
        vec![Relation::Sows {
            animal: 9,
            plant: 0,
        }],
    );
    assert!(of(&sown, 0, Stage::Fruit).paint.fruit.is_some());
    assert!(of(&sown, 0, Stage::Ever).paint.fruit.is_some());
    assert!(of(&sown, 0, Stage::Bloom).paint.fruit.is_none());
    assert!(of(&sown, 1, Stage::Fruit).paint.fruit.is_none());
}

/// Cold gives needles and small leaves, drought gives wax, spines and water storing bodies.
#[test]
fn cold_and_drought_shape_leaves() {
    let cold = Tolerance {
        cold: 240.0,
        heat: 262.0,
        ..MILD
    };
    let dry = Tolerance {
        cold: 295.0,
        heat: 320.0,
        dry: 0.0,
        wet: 0.2,
    };
    let groups = vec![
        producer(1, 0, 0, cold),
        producer(1, 1, 1, dry),
        producer(1, 2, 0, MILD),
    ];
    let life = life(Rung::Skeletal, GREEN, groups, vec![]);
    let (pine, aloe, oak) = (
        of(&life, 0, Stage::Ever),
        of(&life, 1, Stage::Ever),
        of(&life, 2, Stage::Ever),
    );
    assert!(pine.form.needles && pine.form.leaf <= 0.3 && !pine.form.deciduous);
    assert!(aloe.form.wax && aloe.form.spines && aloe.form.store);
    assert!(!oak.form.needles && !oak.form.wax && oak.form.deciduous);
    assert!(oak.form.leaf > pine.form.leaf);
}

/// Richer light gives more canopy layers, and a planet of mats draws every producer as a mat.
#[test]
fn layers_follow_productivity_and_mats_stay_flat() {
    let life_of = |rung| life(rung, GREEN, vec![producer(1, 0, 0, MILD)], vec![]);
    let rich = life_of(Rung::Skeletal);
    let layers = |flux: f32| {
        let w = World {
            flux,
            ..world(9.81, 5800.0)
        };
        dials(&rich, &rich.species[0], &w, 3.0, Stage::Ever)
            .form
            .layers
    };
    assert!(
        layers(0.1) < layers(1.5),
        "{} against {}",
        layers(0.1),
        layers(1.5)
    );
    assert_eq!(of(&life_of(Rung::Mats), 0, Stage::Ever).architecture, MAT);
    assert_eq!(of(&rich, 0, Stage::Ever).architecture, 0);
}

/// The year turns with the season and the hemisphere, and stands still in the tropics and on upright worlds.
#[test]
fn stages_follow_the_season() {
    assert_eq!(stage(0.0, 0.8, 0.4), Stage::Green);
    assert_eq!(stage(0.0, -0.8, 0.4), Stage::Dormant);
    assert_eq!(stage(0.25, 0.8, 0.4), Stage::Fruit);
    assert_eq!(stage(0.75, 0.8, 0.4), Stage::Bloom);
    assert_eq!(stage(0.5, 0.1, 0.4), Stage::Ever);
    assert_eq!(stage(0.5, 0.8, 0.05), Stage::Ever);
    let life = garden(3, &[0, 3], vec![]);
    assert!(of(&life, 0, Stage::Dormant).paint.leaves.is_none());
    assert_ne!(
        of(&life, 0, Stage::Fruit).paint.leaves,
        of(&life, 0, Stage::Green).paint.leaves
    );
}

/// The same life always gives the same dials.
#[test]
fn same_life_same_dials() {
    let pollinated = vec![Relation::Pollinates {
        animal: 4,
        plant: 0,
    }];
    let (a, b) = (
        garden(7, &[0, 5], pollinated.clone()),
        garden(7, &[0, 5], pollinated),
    );
    for i in 0..2 {
        assert_eq!(of(&a, i, Stage::Bloom), of(&b, i, Stage::Bloom));
    }
}
