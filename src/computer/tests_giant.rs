//! Computer tests: the card of a gas giant, of a planet with moons and rings and of a moon,
//! and visits to a moon, with hand-built specs until the universe has them.

use bevy::math::{DVec3, Vec3};

use super::detail::{LINES, brief, readout};
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

/// A gas giant row says so, and its card says it has no ground to land on, its deck pressure,
/// its moons and its rings, in as many lines as any card.
#[test]
fn a_gas_giant_card_says_it_has_no_ground() {
    assert_eq!(brief(Some(&giant())), "gas giant - -150 C");
    let (line, planet) = world(giant());
    let ringed = PlanetSpec {
        moons: 3,
        rings: Some(rings()),
        ..planet.clone()
    };
    let card = readout(&line, Some(&ringed));
    assert_eq!(card.len(), LINES);
    assert_eq!(card[0], "gas giant - -150 C - tilt 0 deg");
    assert_eq!(card[1], "no ground to land on - deck pressure 5.00");
    assert_eq!(card[2], "3 moons - rings 80.0 Mm to 140 Mm");
    assert_eq!(card[4], "radius 6.00 km - gravity 1.00 g");
    let bare = PlanetSpec { moons: 1, ..planet };
    assert_eq!(readout(&line, Some(&bare))[2], "1 moon - no rings");
}

/// A solid planet with moons or rings ends its size line with them, and a moon counts its
/// turn around its planet as its orbit.
#[test]
fn a_solid_planet_names_its_moons_and_a_moon_its_orbit() {
    let (line, planet) = world(PlanetRecord::TEMPERATE);
    let ringed = PlanetSpec {
        moons: 2,
        rings: Some(rings()),
        ..planet.clone()
    };
    let card = readout(&line, Some(&ringed));
    assert_eq!(card[1], "air like home, pressure 1.00");
    assert_eq!(card[4], "radius 6.00 km - gravity 1.00 g - 2 moons - rings");
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
        ..planet
    };
    let card = readout(&line, Some(&moon));
    assert_eq!(card[4], "radius 6.00 km - gravity 1.00 g");
    assert_eq!(card[5], "day 24 min - orbit 3 h 20 min");
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
