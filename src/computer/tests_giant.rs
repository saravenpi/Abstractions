//! Computer tests: the card of a gas giant, of a planet with moons and rings and of a moon,
//! and visits to a moon, with hand-built specs until the universe has them.

use bevy::math::{DVec3, Vec3};

use super::detail::{ROWS, brief, card};
use super::tests_card::fact;
use super::tests_detail::world;
use super::*;
use crate::space::Frame;
use crate::universe::{BodyId, Orbit, PlanetClass, PlanetRecord, PlanetSpec, Rings, Universe};

/// Returns a gas giant record: cold, its deck at five times the home pressure.
fn giant() -> PlanetRecord {
    PlanetRecord {
        class: PlanetClass::GasGiant,
        temperature: 123.15,
        pressure: 5.0,
        ..PlanetRecord::TEMPERATE
    }
}

/// Returns rings from 80 Mm to 140 Mm.
fn rings() -> Rings {
    Rings {
        inner: 8.0e7,
        outer: 1.4e8,
        density: 0.5,
        color: Vec3::ONE,
    }
}

/// A gas giant row says so, and its card says it is a gas giant with no ground to land on,
/// its deck pressure, its moons and its rings.
#[test]
fn a_gas_giant_card_says_it_has_no_ground() {
    assert_eq!(brief(Some(&giant())), "gas giant - -150 C");
    let universe = Universe::new(7);
    let (line, planet) = world(giant());
    let ringed = PlanetSpec {
        moons: 3,
        rings: Some(rings()),
        ..planet.clone()
    };
    let shown = card(&line, Some(&ringed), &universe);
    assert!(
        shown.subtitle.starts_with("Gas giant of "),
        "{}",
        shown.subtitle
    );
    assert_eq!(fact(&shown, "Ground"), Some("none to land on"));
    assert_eq!(fact(&shown, "Deck pressure"), Some("5.00"));
    assert_eq!(fact(&shown, "Moons"), Some("3"));
    assert_eq!(fact(&shown, "Rings"), Some("80.0 Mm to 140 Mm"));
    assert_eq!(fact(&shown, "Radius"), Some("6.00 km"));
    assert!(shown.rows.len() <= ROWS);
    let bare = card(&line, Some(&PlanetSpec { moons: 1, ..planet }), &universe);
    assert_eq!(
        (fact(&bare, "Moons"), fact(&bare, "Rings")),
        (Some("1"), None)
    );
}

/// A solid planet with moons or rings counts them on its card, one with neither says
/// nothing of them, and a moon says what it turns around and counts that turn as its orbit.
#[test]
fn a_solid_planet_names_its_moons_and_a_moon_its_orbit() {
    let universe = Universe::new(7);
    let (line, planet) = world(PlanetRecord::TEMPERATE);
    let ringed = PlanetSpec {
        moons: 2,
        rings: Some(rings()),
        ..planet.clone()
    };
    let shown = card(&line, Some(&ringed), &universe);
    assert_eq!(fact(&shown, "Air"), Some("like home, pressure 1.00"));
    assert_eq!(fact(&shown, "Moons"), Some("2"));
    assert!(fact(&shown, "Rings").is_some());
    let moon = PlanetSpec {
        id: BodyId {
            moon: 1,
            ..planet.id
        },
        host: Some(Orbit {
            radius: 1.0e11,
            seconds: 1.0e6,
            phase: 0.0,
        }),
        moons: 0,
        ..planet
    };
    let shown = card(&line, Some(&moon), &universe);
    let host = universe.planet(moon.id.planet()).name;
    assert_eq!(shown.subtitle, format!("Island moon of {host}"));
    assert_eq!(fact(&shown, "Moons"), None);
    assert_eq!(fact(&shown, "Orbit"), Some("3 h 20 min"));
    assert_eq!(fact(&shown, "Year"), None);
}

/// Standing on a moon is being inside its system and its galaxy, so a visit charts the system
/// of its planet, and the atlas keeps the moon's own id and nothing else for it.
#[test]
fn visiting_a_moon_visits_its_system() {
    let universe = Universe::new(7);
    let mut atlas = Atlas::default();
    let moon = BodyId {
        moon: 1,
        ..universe.home()
    };
    let (body, star, galaxy) = whereabouts(&universe, Frame::Surface(moon), DVec3::ZERO);
    assert_eq!(
        (body, star, galaxy),
        (Some(moon), Some(moon.star), Some(moon.star.galaxy))
    );
    let inside = [
        Target::Body(moon),
        Target::Star(moon.star),
        Target::Galaxy(moon.star.galaxy),
    ];
    assert!(inside.into_iter().all(|target| atlas.visit(target)));
    assert!(!atlas.visited(Target::Body(moon.planet())));
    assert_eq!(atlas.bodies, [moon]);
    assert!(toml::to_string(&atlas).unwrap().contains("moon = 1"));
}
