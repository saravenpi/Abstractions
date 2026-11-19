//! Tests of guilds, habitats, activity and glow.

use super::*;
use crate::life::tissue::tests::setting;
use crate::life::{Solvent, Structure};

/// Plant eaters never hunt, high levels always do, and soft life never hunts at all.
#[test]
fn guild_follows_the_level() {
    let old = setting(Rung::Old, Structure::Carbon, Solvent::Water, 0.5);
    let soft = setting(Rung::Soft, Structure::Carbon, Solvent::Water, 0.5);
    let mut rng = Rng::new(1);
    for _ in 0..200 {
        assert!(!matches!(
            of(2.0, 0.3, &old, &mut rng),
            Guild::Hunter | Guild::Ambusher
        ));
        assert!(matches!(
            of(3.5, 0.9, &old, &mut rng),
            Guild::Hunter | Guild::Ambusher
        ));
        assert!(!matches!(
            of(3.5, 0.9, &soft, &mut rng),
            Guild::Hunter | Guild::Ambusher
        ));
    }
}

/// No swimmers on a dry world, no flyers in thin air.
#[test]
fn habitat_follows_the_planet() {
    let mut dry = setting(Rung::Old, Structure::Carbon, Solvent::Water, 0.0);
    dry.world.pressure = 0.1;
    let mut rng = Rng::new(2);
    for guild in [Guild::Filter, Guild::Nectar, Guild::Hunter, Guild::Grazer] {
        for _ in 0..50 {
            assert_eq!(habitat(guild, &dry, &mut rng), Habitat::Ground);
        }
    }
}

/// Under a dim red star most animals are out at night, under a yellow one most by day.
#[test]
fn dim_stars_make_night_life() {
    let sun = setting(Rung::Old, Structure::Carbon, Solvent::Water, 0.5);
    let mut dim = sun;
    dim.world.star = 3000.0;
    let nights = |s: &Setting| {
        let mut rng = Rng::new(3);
        (0..400)
            .filter(|_| activity(Guild::Grazer, Habitat::Ground, s, &mut rng) == Activity::Night)
            .count()
    };
    assert!(nights(&dim) > 200 && nights(&sun) < 120);
}

/// A night ambusher glows as a lure in the colour of the planet's chemistry, a day grazer never glows.
#[test]
fn glow_pays_only_in_the_dark() {
    let at = setting(Rung::Old, Structure::Carbon, Solvent::Water, 0.5);
    let mut rng = Rng::new(4);
    let mut role = crate::life::tissue::tests::role(Guild::Ambusher, 3.0);
    role.activity = Activity::Night;
    let lure = glow(&role, &[], &at, &mut rng).map(|g| (g.purpose, g.colour));
    assert_eq!(lure, Some((GlowUse::Lure, at.chemistry.glow)));
    let grazer = crate::life::tissue::tests::role(Guild::Grazer, 2.0);
    assert!(glow(&grazer, &[], &at, &mut rng).is_none());
}
