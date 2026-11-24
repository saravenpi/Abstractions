//! Frame conversion: a point fixed on a turning planet, a ship orbiting a planet a little in
//! the past, and a ship in deep space, each sent in its wire frame and drawn in another.

use abstractions_net::jitter::{Buffer, Snap};
use abstractions_net::wire::{self, Body, ShipBody};
use bevy::math::{DQuat, DVec3, Quat, Vec3};

use crate::net::state::bodies::Bodies;
use crate::net::state::frame::{Place, Pose, velocity};
use crate::universe::{PlanetSpec, Universe};
use crate::world::Link;

/// Seed of the universe of these tests.
const SEED: u64 = 42;

/// Returns the universe of these tests, its home planet and bodies to convert against.
fn world() -> (Universe, PlanetSpec, Bodies) {
    let universe = Universe::new(SEED);
    let mut bodies = Bodies::default();
    bodies.sync(&universe);
    (universe, universe.planet(universe.home()), bodies)
}

/// Returns a pose at `pos` moving at `vel`, not turned.
fn at(pos: DVec3, vel: DVec3) -> Pose {
    Pose {
        pos,
        vel,
        rot: DQuat::IDENTITY,
    }
}

/// Returns a ship body at `pos` moving at `vel`, packed for `frame` and read back as the wire
/// carries it.
fn wired(frame: wire::Frame, pos: DVec3, vel: DVec3) -> ShipBody {
    let body = ShipBody {
        position: pos,
        velocity: vel.as_vec3(),
        ..ShipBody::default()
    };
    let mut bytes = Vec::new();
    body.encode(&frame, &mut bytes);
    ShipBody::decode(&frame, &bytes).unwrap()
}

/// Returns a snapshot sampled at `time` at `pos` moving at `vel`.
fn snap(time: f64, pos: DVec3, vel: Vec3) -> Snap<()> {
    Snap {
        time,
        position: pos,
        velocity: Some(vel),
        rotation: Quat::IDENTITY,
        extra: (),
    }
}

/// A point on the ground, taken in the star frame, sent in the surface frame and drawn in the
/// star frame at later times, stays where the planet itself carries it, within 1 mm.
#[test]
fn a_point_on_a_turning_planet_turns_with_it() {
    let (_, spec, mut bodies) = world();
    let (home, star) = (Place::Surface(spec.id), Place::System(spec.id.star));
    let spot = DVec3::new(0.3, 0.8, -0.5).normalize() * spec.radius;
    let frame = bodies.choose((home, spot), true, 100.0, None).unwrap();
    assert_eq!(frame, wire::Frame::Surface(spec.id));
    let sent = at(Link::orbiting(&spec, 100.0).to_anchor(spot), DVec3::ZERO);
    let wire = bodies.convert(sent, star, home, 100.0).unwrap();
    let body = wired(frame, wire.pos, DVec3::ZERO);
    assert!(body.position.distance(spot) < 1e-3);
    let ground = at(body.position, DVec3::ZERO);
    for time in [100.0, 1234.5, 86_400.0, 3.0e6] {
        let drawn = bodies.convert(ground, home, star, time).unwrap();
        let off = drawn
            .pos
            .distance(Link::orbiting(&spec, time).to_anchor(spot));
        assert!(off < 1e-3, "{off} m off at {time} s");
        let same = bodies.convert(ground, home, home, time).unwrap();
        assert_eq!(same.pos, body.position);
    }
}

/// Returns where a ship on a circular orbit 1.5 radii from the centre of `spec` is `time`
/// seconds in, and how fast it goes, relative to the planet in star axes.
fn orbit(spec: &PlanetSpec, time: f64) -> (DVec3, DVec3) {
    let radius = 1.5 * spec.radius;
    let speed = (spec.mu() / radius).sqrt();
    let (sin, cos) = (speed / radius * time).sin_cos();
    let along = DVec3::new(-sin, 0.0, cos);
    (DVec3::new(cos, 0.0, sin) * radius, along * speed)
}

/// A ship in orbit is sent around its planet and drawn 100 ms in the past, yet within 1 m of
/// where it was then relative to the planet, which moves kilometres per second.
#[test]
fn an_orbiting_ship_keeps_to_its_planet() {
    let (_, spec, mut bodies) = world();
    let star = Place::System(spec.id.star);
    let mut buffer = Buffer::<()>::new(0.05);
    for step in 0..90 {
        let time = 5000.0 + f64::from(step) / 30.0;
        let (rel, vel) = orbit(&spec, time);
        let truth = at(spec.position(time) + rel, velocity(&spec, time) + vel);
        let frame = bodies.choose((star, truth.pos), false, time, None).unwrap();
        assert_eq!(frame, wire::Frame::Around(spec.id));
        let wire = bodies.convert(truth, star, frame.into(), time).unwrap();
        let body = wired(frame, wire.pos, wire.vel);
        buffer.push(snap(time, body.position, body.velocity), time);
        let Some(view) = buffer.sample(time).filter(|_| step > 10) else {
            continue;
        };
        assert!(time - view.time > 0.099);
        let seen = at(view.position, DVec3::ZERO);
        let drawn = bodies.convert(seen, frame.into(), star, time).unwrap();
        let off = (drawn.pos - spec.position(time)).distance(orbit(&spec, view.time).0);
        assert!(off < 1.0, "{off} m off the orbit");
    }
}

/// A ship far from every body is sent in the frame of its star and drawn where it was, in the
/// star frame and in the frame of the galaxy.
#[test]
fn a_ship_in_deep_space_goes_by_its_star() {
    let (universe, spec, mut bodies) = world();
    let (star, galaxy) = (
        Place::System(spec.id.star),
        Place::Galaxy(spec.id.star.galaxy),
    );
    let centre = universe.star(spec.id.star).unwrap().pos;
    let speed = DVec3::new(3000.0, 0.0, 500.0);
    let path = |time: f64| DVec3::new(2.0e9, 1.0e7, 0.0) + speed * time;
    let mut buffer = Buffer::<()>::new(1.0 / 30.0);
    for step in 0..60 {
        let time = 100.0 + f64::from(step) / 30.0;
        let frame = bodies.choose((star, path(time)), false, time, None);
        assert_eq!(frame, Some(wire::Frame::System(spec.id.star)));
        buffer.push(snap(time, path(time), speed.as_vec3()), time);
        let Some(view) = buffer.sample(time).filter(|_| step > 5) else {
            continue;
        };
        assert!(view.position.distance(path(view.time)) < 1e-3);
        let seen = at(view.position, DVec3::ZERO);
        let drawn = bodies.convert(seen, star, galaxy, time).unwrap();
        assert!(drawn.pos.distance(centre + path(view.time)) < 1e-3);
    }
    let between = bodies.choose((galaxy, centre), false, 0.0, Some(spec.id.star));
    assert_eq!(between, Some(wire::Frame::System(spec.id.star)));
}

/// A ship entering another galaxy with no prior star there still picks a reference star frame.
#[test]
fn a_ship_in_another_galaxy_picks_a_reference_star() {
    let (universe, spec, mut bodies) = world();
    let home = spec.id.star.galaxy;
    let other = universe
        .galaxies_near(home, 2)
        .into_iter()
        .find(|g| g.id != home)
        .unwrap();
    let picked = bodies.choose((Place::Galaxy(other.id), DVec3::ZERO), false, 0.0, None);
    assert!(matches!(picked, Some(wire::Frame::System(star)) if star.galaxy == other.id));
}
