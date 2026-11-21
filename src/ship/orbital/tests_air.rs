//! Tests of the floor of the orbital cruise: it never goes into the air, and a pilot who asks to
//! go lower is left at rest over the ground, where no entry heats the nose.

use bevy::math::{DQuat, DVec3};

use super::near::Over;
use super::rig::{DT, band, earth, skim_at};
use super::skim::{Ask, step};
use super::{Note, Orbital, Regime};
use crate::render::cool_speed;
use crate::ship::body::Ship;
use crate::ship::drag::density;
use crate::space::Frame;

/// Held down at full lever from 150 km, the cruise comes down at 2 km/s to its floor above the
/// air and no lower, and asks to end there within half a minute.
#[test]
fn the_cruise_never_enters_the_air() {
    let planet = earth();
    let band = band(&planet);
    let mut skim = skim_at(&planet, 150_000.0);
    let ask = Ask {
        throttle: 1.0,
        climb: -1.0,
        ..Ask::default()
    };
    let (mut time, mut ended) = (0.0, false);
    while !ended && time < 60.0 {
        let stride = step(&skim, &band, &ask, DT);
        let height = stride.skim.pos.length() - planet.radius;
        assert!(height >= band.floor - 1e-6, "{height} m at {time} s");
        assert!(height > planet.atmosphere_height);
        (skim, ended, time) = (stride.skim, stride.floor, time + DT);
    }
    assert!(ended && time < 30.0, "ended {ended} after {time} s");
    assert!(
        skim.speed > 100_000.0,
        "the cruise slowed to {} m/s",
        skim.speed
    );
}

/// The cruise that ends at its floor leaves the ship at rest against the turning ground under
/// it, however fast it flew: the air at the top of the atmosphere cannot heat a ship that does
/// not move through it.
#[test]
fn the_end_of_a_cruise_leaves_the_ship_at_rest_over_the_ground() {
    let planet = earth();
    let height = band(&planet).floor;
    let spin = DVec3::Y * crate::ship::frames::spin_rate(&planet);
    let over = Over {
        planet: planet.clone(),
        centre: DVec3::new(1.5e11, 0.0, 0.0),
        vel: DVec3::new(0.0, 0.0, -29_800.0),
        turn: DQuat::IDENTITY,
        spin,
    };
    let rel = DVec3::X * (planet.radius + height);
    let mut ship = Ship::at_rest(
        Frame::System(planet.id.star),
        over.centre + rel,
        DQuat::IDENTITY,
    );
    ship.vel = over.vel + DVec3::NEG_Z * 226_000.0;
    let mut orbital = Orbital {
        regime: Regime::Cruise,
        ..Orbital::default()
    };
    orbital.rest(&mut ship, &over, Some(Note::Floor));
    let airspeed = (ship.vel - over.vel - spin.cross(rel)).length();
    let top = density(&planet, planet.atmosphere_height - 1.0);
    assert!(
        airspeed < 1.0e-6 && airspeed < cool_speed(top),
        "{airspeed} m/s"
    );
    assert_eq!(orbital.regime(), Regime::Off);
    assert_eq!(orbital.note(), Some(Note::Floor));
}
