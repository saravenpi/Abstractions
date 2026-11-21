//! Tests of flying home through the cruise: the home key takes the base as the target, which
//! the ship cruises to at once above the air, climbs out of the air to when it is far, warps to
//! when the planet is away, and leaves to the marker when it is close.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::home::{Move, Spot, answer, apply, on_homeward, spot};
use super::near::Over;
use super::rig::earth;
use super::skim::Band;
use super::{Note, Orbital, Pending, Regime};
use crate::computer::{Destination, Homeward, Target};
use crate::home::Base;
use crate::ship::body::Ship;
use crate::ship::pilot::Autopilot;
use crate::space::Frame;
use crate::universe::{BodyId, PlanetSpec};

/// Returns a base on the equator of `planet` at +X.
fn base(planet: &PlanetSpec) -> Base {
    let ground = DVec3::X * (planet.radius + 50.0);
    Base {
        body: planet.id,
        centre: ground,
        up: DVec3::X,
        east: DVec3::NEG_Z,
        north: DVec3::Y,
        wake: ground + DVec3::Y * 3.0,
        pad: ground + DVec3::Z * 12.0,
        heading: DVec3::Y,
        door: ground + DVec3::Y * 2.0,
    }
}

/// Returns the planet as seen from its own surface frame.
fn below(planet: &PlanetSpec) -> Over {
    Over {
        planet: planet.clone(),
        centre: DVec3::ZERO,
        vel: DVec3::ZERO,
        turn: DQuat::IDENTITY,
        spin: DVec3::ZERO,
    }
}

/// Returns a target at the base on `planet`, not yet flown to.
fn asked(planet: &PlanetSpec) -> Pending {
    Pending {
        body: planet.id,
        goal: base(planet).home(),
        flown: false,
    }
}

/// The home key on the planet of the base takes the base as the target to cruise to.
#[test]
fn the_home_key_takes_the_base_as_the_target() {
    let planet = earth();
    let mut app = App::new();
    app.init_resource::<Orbital>()
        .insert_resource(base(&planet))
        .add_observer(on_homeward);
    app.world_mut().trigger(Homeward(planet.id));
    let pending = app.world().resource::<Orbital>().pending;
    assert_eq!(pending, Some(asked(&planet)));
    let moon = BodyId {
        moon: 1,
        ..planet.id
    };
    app.world_mut().trigger(Homeward(moon));
    let kept = app.world().resource::<Orbital>().pending;
    assert_eq!(kept, Some(asked(&planet)), "a moon has no base");
}

/// Above the air the target starts the cruise to the base at once, ends the course to its planet
/// and takes the ship from the autopilot.
#[test]
fn above_the_air_home_starts_the_cruise_to_the_base() {
    let planet = earth();
    let over = below(&planet);
    let pending = asked(&planet);
    assert_eq!(answer(&pending, Spot::Over, None), Move::Cruise);
    let height = Band::of(&planet).floor + 200_000.0;
    let pos = DVec3::NEG_X * (planet.radius + height);
    let mut ship = Ship::at_rest(Frame::System(planet.id.star), pos, DQuat::IDENTITY);
    let (mut orbital, mut autopilot) = (Orbital::default(), Autopilot::default());
    let mut course = Destination(Some(Target::Body(planet.id)));
    autopilot.engage(Target::Body(planet.id));
    let wheel = (&mut orbital, &mut autopilot, &mut course);
    apply(Move::Cruise, pending, wheel, &mut ship, Some(&over));
    assert_eq!(orbital.regime(), Regime::Cruise);
    assert_eq!(orbital.goal, Some(pending.goal));
    assert!((orbital.height - height).abs() < 1.0, "{}", orbital.height);
    assert_eq!((autopilot.target(), course.0), (None, None));
    assert_eq!(ship.throttle, 1.0);
}

/// In the air the base far away is a climb to the cruise, close by it is left to the marker, and
/// away from the planet it is a course to it, cruised from on arrival.
#[test]
fn from_the_air_or_from_afar_home_climbs_or_sets_the_course() {
    let planet = earth();
    let (over, pending) = (below(&planet), asked(&planet));
    let mut ship = Ship::at_rest(Frame::Surface(planet.id), DVec3::X, DQuat::IDENTITY);
    for (dir, far) in [(DVec3::NEG_X, true), (DVec3::new(1.0, 0.001, 0.0), false)] {
        ship.pos = dir.normalize() * (planet.radius + 2000.0);
        assert_eq!(spot(&ship, Some(&over), &pending, false), Spot::Air { far });
    }
    assert_eq!(answer(&pending, Spot::Air { far: true }, None), Move::Climb);
    let near = answer(&pending, Spot::Air { far: false }, None);
    assert_eq!(near, Move::Drop(Some(Note::Near)));
    assert_eq!(spot(&ship, None, &pending, false), Spot::Away);
    assert_eq!(answer(&pending, Spot::Away, None), Move::Route);
    let home = Some(Target::Body(planet.id));
    assert_eq!(answer(&pending, Spot::Away, home), Move::Follow);
    let flown = Pending {
        flown: true,
        ..pending
    };
    assert_eq!(answer(&flown, Spot::Away, home), Move::Wait);
    assert_eq!(answer(&flown, Spot::Busy, None), Move::Wait);
    assert_eq!(answer(&flown, Spot::Away, None), Move::Drop(None));
}
