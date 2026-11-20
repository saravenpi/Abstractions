//! Tests of how fast a plant sheds: calm summer is almost nothing, wind and autumn and rain raise it.

use super::*;
use crate::effects::rig::{home, plant};

/// In calm summer air a healthy tree loses almost nothing: under one leaf in a minute.
#[test]
fn calm_summer_is_almost_nothing() {
    let rate = leaves(&plant(true, Stage::Green, false), &home(1.0, 0.0));
    assert!(rate < 1.0 / 60.0, "{rate}");
}

/// The rate grows with the wind and steeply past the threshold: it is flat under it.
#[test]
fn wind_raises_the_rate() {
    let tree = plant(true, Stage::Green, false);
    let at = |speed| leaves(&tree, &home(speed, 0.0));
    assert_eq!(at(1.0), at(3.0));
    assert!(at(4.0) > at(3.0) && at(6.0) > 4.0 * at(4.0));
}

/// Thin air pushes less than thick air at the same speed, and no air pushes nothing.
#[test]
fn the_air_sets_the_drag() {
    let tree = plant(true, Stage::Fruit, false);
    let thin = Air {
        thickness: 0.3,
        ..home(6.0, 0.0)
    };
    let void = Air {
        thickness: 0.0,
        ..home(6.0, 0.0)
    };
    assert!(leaves(&tree, &thin) < leaves(&tree, &home(6.0, 0.0)));
    assert_eq!(leaves(&tree, &void), 0.0);
}

/// A deciduous tree sheds far more in autumn than in summer, in calm air and in wind, and nothing when bare.
#[test]
fn autumn_sheds_and_winter_is_bare() {
    for (speed, times) in [(1.0, 20.0), (5.0, 8.0)] {
        let summer = leaves(&plant(true, Stage::Green, false), &home(speed, 0.0));
        let autumn = leaves(&plant(true, Stage::Fruit, false), &home(speed, 0.0));
        assert!(
            autumn > times * summer,
            "{speed}: {autumn} against {summer}"
        );
    }
    let mut bare = plant(true, Stage::Dormant, false);
    bare.leaf = None;
    assert_eq!(leaves(&bare, &home(8.0, 1.0)), 0.0);
}

/// An evergreen loses leaves at the same slow rate in every season.
#[test]
fn an_evergreen_is_steady() {
    let rates = [Stage::Bloom, Stage::Green, Stage::Fruit, Stage::Dormant]
        .map(|stage| leaves(&plant(false, stage, false), &home(1.0, 0.0)));
    assert!(rates.iter().all(|&r| r == rates[0] && r > 0.0 && r < 0.05));
}

/// Without seasons only the wind and the thirst of the plant matter.
#[test]
fn a_planet_without_seasons_follows_the_wind() {
    let tree = plant(true, Stage::Ever, false);
    let thirsty = Shed {
        stress: 1.0,
        ..tree
    };
    assert!(leaves(&tree, &home(1.0, 0.0)) < 0.05);
    assert!(leaves(&tree, &home(6.0, 0.0)) > leaves(&tree, &home(1.0, 0.0)));
    assert!(leaves(&thirsty, &home(1.0, 0.0)) > leaves(&tree, &home(1.0, 0.0)));
}

/// Rain knocks leaves down, and a storm of rain and wind more.
#[test]
fn rain_raises_the_rate() {
    let tree = plant(true, Stage::Green, false);
    let dry = leaves(&tree, &home(2.0, 0.0));
    let wet = leaves(&tree, &home(2.0, 1.0));
    let storm = leaves(&tree, &home(6.0, 1.0));
    assert!(wet > 5.0 * dry && storm > wet);
}

/// Petals come only while the plant is in flower, more in wind and rain; pollen only when dry and breezy.
#[test]
fn petals_and_pollen_follow_the_flowering() {
    let bare = plant(true, Stage::Green, false);
    let bloom = plant(true, Stage::Bloom, true);
    assert_eq!(
        (
            petals(&bare, &home(6.0, 1.0)),
            pollen(&bare, &home(3.0, 0.0))
        ),
        (0.0, 0.0)
    );
    assert!(petals(&bloom, &home(6.0, 0.0)) > petals(&bloom, &home(1.0, 0.0)));
    assert!(petals(&bloom, &home(1.0, 1.0)) > petals(&bloom, &home(1.0, 0.0)));
    assert_eq!(pollen(&bloom, &home(3.0, 0.5)), 0.0);
    assert_eq!(pollen(&bloom, &home(0.5, 0.0)), 0.0);
    assert!(pollen(&bloom, &home(3.0, 0.0)) > 0.0);
}
