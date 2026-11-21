//! Tests of where the jets of a ship meet the ground and what the ground gives: dust is raised
//! under a jet within its reach of dry ground, a full cloud off loose soil and a thin grit off
//! rock and ice, and none off water.

use bevy::math::{DQuat, DVec3};

use super::super::Exhaust;
use super::jet::{ENGINE, Hit, JETS, Jet, NOZZLE, impinge, jets, reach};
use super::soil::loose;
use super::tune::{GRIT, REACH};
use crate::avatar::Soil;
use crate::space::Placed;
use crate::universe::PlanetClass;
use crate::world::Surface;

/// Sea level radius of the test planet in metres.
const RADIUS: f64 = 1.0e6;

/// Share of its full push a lift nozzle gives in a hover under standard gravity.
const HOVER: f32 = 0.4;

/// Returns the ground of a smooth planet at sea level, in every direction.
fn flat(_: DVec3) -> f64 {
    RADIUS
}

/// Returns a lift nozzle in a hover `height` metres over the flat ground, blowing straight down.
fn nozzle(height: f64) -> Jet {
    Jet {
        from: DVec3::Y * (RADIUS + height),
        along: DVec3::NEG_Y,
        push: NOZZLE * HOVER,
    }
}

/// Returns where a hovering lift nozzle `height` metres up meets dry ground of `surface`, and
/// what that ground gives with its share of a full cloud: dust only when both hold.
fn raised(height: f64, surface: Surface) -> Option<(Hit, (Soil, f32))> {
    let hit = impinge(&nozzle(height), flat)?;
    Some((hit, loose(PlanetClass::Island, surface, 20.0)?))
}

/// The reach of a jet grows with the root of its thrust, from [`REACH`] at the full push of a
/// lift nozzle, and a main engine reaches farther than a lift nozzle.
#[test]
fn the_reach_grows_with_the_root_of_the_thrust() {
    assert!((reach(NOZZLE) - REACH).abs() < 1e-3);
    assert!((reach(4.0 * NOZZLE) - 2.0 * REACH).abs() < 1e-2);
    assert!(reach(ENGINE) > reach(NOZZLE));
    assert_eq!(reach(0.0), 0.0);
}

/// A hovering lift nozzle meets the ground right under it at 5 and 15 metres, harder the lower
/// it is, and not at all 40 metres up, past its reach.
#[test]
fn a_hover_meets_the_ground_only_within_its_reach() {
    let low = impinge(&nozzle(5.0), flat).expect("5 m");
    let mid = impinge(&nozzle(15.0), flat).expect("15 m");
    assert!((low.at - DVec3::Y * RADIUS).length() < 1e-6, "{:?}", low.at);
    assert!((low.run - 5.0).abs() < 1e-3 && (mid.run - 15.0).abs() < 1e-3);
    assert!(low.strength > mid.strength && mid.strength > 0.0);
    assert!(impinge(&nozzle(40.0), flat).is_none());
}

/// A jet that points up, along the ground, or starts under it meets nothing; a slanted jet meets
/// the ground ahead of its nozzle, farther along its axis than straight down.
#[test]
fn only_a_jet_that_points_down_meets_the_ground() {
    let up = Jet {
        along: DVec3::Y,
        ..nozzle(5.0)
    };
    let level = Jet {
        along: DVec3::X,
        ..nozzle(5.0)
    };
    let buried = nozzle(-1.0);
    assert!(
        [up, level, buried]
            .iter()
            .all(|jet| impinge(jet, flat).is_none())
    );
    let slant = Jet {
        along: DVec3::new(1.0, -1.0, 0.0).normalize(),
        ..nozzle(5.0)
    };
    let hit = impinge(&slant, flat).expect("a slanted jet");
    assert!(hit.at.x > 4.0 && hit.run > 6.0, "{hit:?}");
}

/// A low jet raises a full cloud off sand, the earth under grass and snow on land, a thin grit
/// off rock and the ice of a frozen sea, and none of them out of its reach. The sea, the shallows,
/// a reef, a river and a pond give no dust, nor does the cloud deck of a gas giant.
#[test]
fn dust_rises_under_a_jet_within_reach_of_dry_ground() {
    for (surface, soil, share) in [
        (Surface::Sand, Soil::Sand, 1.0),
        (Surface::Grass, Soil::Earth, 1.0),
        (Surface::Snow, Soil::Snow, 1.0),
        (Surface::Rock, Soil::Rock, GRIT.0),
    ] {
        let got = raised(5.0, surface).map(|(_, given)| given);
        assert_eq!(got, Some((soil, share)), "{surface:?}");
        assert!(raised(40.0, surface).is_none(), "{surface:?} at 40 m");
    }
    let wet = [
        Surface::Water,
        Surface::Shallows,
        Surface::River,
        Surface::Pond,
        Surface::Reef,
    ];
    assert!(wet.iter().all(|surface| raised(5.0, *surface).is_none()));
    let ice = loose(PlanetClass::Ice, Surface::Snow, -3.0);
    assert_eq!(ice, Some((Soil::Snow, GRIT.0)));
    assert_eq!(loose(PlanetClass::GasGiant, Surface::Sand, 0.0), None);
    let bare = loose(PlanetClass::Barren, Surface::Rock, 5.0);
    assert_eq!(bare, Some((Soil::Land, GRIT.0)));
}

/// A level ship blows its lift nozzles down under its belly and its main engines back behind
/// its tail, each with its share of the thrust of the ship.
#[test]
fn a_level_ship_blows_down_and_back() {
    let placed = Placed {
        pos: DVec3::Y * (RADIUS + 10.0),
        rot: DQuat::IDENTITY,
    };
    let exhaust = Exhaust {
        lift: HOVER,
        main: 1.0,
        ..Exhaust::default()
    };
    let all = jets(&placed, &exhaust);
    assert_eq!(all.len(), JETS);
    assert!(
        all[..4]
            .iter()
            .all(|jet| jet.along == DVec3::NEG_Y && jet.push == NOZZLE * HOVER)
    );
    assert!(
        all[4..]
            .iter()
            .all(|jet| jet.along == DVec3::Z && jet.push == ENGINE)
    );
    assert!(all[..4].iter().all(|jet| jet.from.y < RADIUS + 10.0));
}
