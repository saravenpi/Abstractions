//! The frame of a flying ship near the edge of the reach of a planet, and a ship hovering over
//! a turning planet drawn on its ground a little in the past.

use abstractions_net::jitter::{Buffer, Snap};
use abstractions_net::wire;
use bevy::math::{DQuat, DVec3, Quat};

use crate::net::state::bodies::{Bodies, ENTER, REACH};
use crate::net::state::draw::moment;
use crate::net::state::frame::{Place, Pose};
use crate::universe::Universe;

/// Returns a pose at `pos` moving at `vel`, not turned.
fn at(pos: DVec3, vel: DVec3) -> Pose {
    Pose {
        pos,
        vel,
        rot: DQuat::IDENTITY,
    }
}

/// A ship flying in from deep space goes around the planet only inside 95 radii, stays around
/// it out to 100 radii and goes back to the frame of the star beyond.
#[test]
fn the_reach_of_a_planet_has_two_edges() {
    let universe = Universe::new(42);
    let spec = universe.planet(universe.home());
    let mut bodies = Bodies::default();
    bodies.sync(&universe);
    let (star, time) = (Place::System(spec.id.star), 500.0);
    let mut frame = |radii: f64| {
        let pos = spec.position(time) + DVec3::Y * radii * spec.radius;
        bodies.choose((star, pos), false, time, None)
    };
    let (around, out) = (
        Some(wire::Frame::Around(spec.id)),
        Some(wire::Frame::System(spec.id.star)),
    );
    let between = (ENTER + REACH) / 2.0;
    assert_eq!(frame(between), out);
    assert_eq!(frame(ENTER - 1.0), around);
    assert_eq!(frame(between), around);
    assert_eq!(frame(REACH + 1.0), out);
    assert_eq!(frame(between), out);
}

/// A ship hovering over one spot of a turning planet is sent around the planet and drawn on
/// its ground 100 ms in the past, yet stays over that spot within 1 cm.
#[test]
fn a_hovering_ship_stays_over_its_ground() {
    let universe = Universe::new(42);
    let spec = universe.planet(universe.home());
    let mut bodies = Bodies::default();
    bodies.sync(&universe);
    let (ground, around) = (Place::Surface(spec.id), Place::Around(spec.id));
    let spot = DVec3::new(0.3, 0.8, -0.5).normalize() * (spec.radius + 300.0);
    let mut buffer = Buffer::<()>::new(1.0 / 30.0);
    for step in 0..90 {
        let time = 2000.0 + f64::from(step) / 30.0;
        let sent = bodies
            .convert(at(spot, DVec3::ZERO), ground, around, time)
            .unwrap();
        let snap = Snap {
            time,
            position: sent.pos,
            velocity: Some(sent.vel.as_vec3()),
            rotation: Quat::IDENTITY,
            extra: (),
        };
        buffer.push(snap, time);
        let Some(view) = buffer.sample(time).filter(|_| step > 10) else {
            continue;
        };
        let when = moment((around, ground), view.time, time);
        let drawn = bodies.convert(at(view.position, DVec3::ZERO), around, ground, when);
        let off = drawn.unwrap().pos.distance(spot);
        assert!(off < 0.01, "{off} m off the spot");
    }
}
