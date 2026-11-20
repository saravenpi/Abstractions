//! Tests of the scaling laws of populations: density falls as `mass^-0.75` (Damuth), home range area
//! grows as `mass^1` (Jetz and others 2004), grazer herds grow with body size, and grazers keep to
//! open land.

use super::{Site, density};
use crate::life::herd::fixtures::{homeworld, role, site, species};
use crate::life::{Activity, Guild, Habitat, Rung, history};
use crate::world::Biome;

/// Returns a meadow at 50 m in full daylight, or `biome` there.
fn on(biome: Biome) -> Site {
    Site {
        biome,
        ..site(50.0, 1.0)
    }
}

/// A grazer ten times heavier lives ten to the 0.75 times sparser, and a hunter one level up a tenth
/// as dense as a grazer of its mass.
#[test]
fn density_follows_damuth() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let hunter = role(Guild::Hunter, Habitat::Ground, Activity::Day, 3.0);
    let at = |r, mass| density(&species(1, r, mass), &on(Biome::Meadow));
    let ratio = at(grazer, 10.0) / at(grazer, 100.0);
    assert!((ratio - 10f32.powf(0.75)).abs() < 0.01, "{ratio}");
    let level = at(hunter, 100.0) / at(grazer, 100.0);
    assert!((level - 0.1).abs() < 1e-3, "{level}");
}

/// Home range area grows as mass: a grazer four times heavier ranges twice as far, give or take the
/// tenth of jitter of each.
#[test]
fn home_range_area_grows_as_mass() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let range = |mass| history::of(3, &grazer, mass, Rung::Skeletal, &homeworld()).range;
    for mass in [10.0, 100.0, 1000.0] {
        let ratio = range(4.0 * mass) / range(mass);
        assert!((1.6..2.5).contains(&ratio), "{mass} kg: {ratio}");
    }
}

/// Large grazers live in herds of 5 to 30, and keep to open meadows over woods.
#[test]
fn large_grazers_herd_on_open_land() {
    let grazer = role(Guild::Grazer, Habitat::Ground, Activity::Day, 2.0);
    let big = species(1, grazer, 200.0);
    assert_eq!((big.history.group_min, big.history.group_max), (5, 30));
    let (meadow, forest) = (
        density(&big, &on(Biome::Meadow)),
        density(&big, &on(Biome::Forest)),
    );
    assert!(meadow > 2.0 * forest, "{meadow} vs {forest}");
    let browser = species(
        2,
        role(Guild::Browser, Habitat::Ground, Activity::Day, 2.0),
        200.0,
    );
    assert!(density(&browser, &on(Biome::Forest)) > density(&browser, &on(Biome::Meadow)));
}
