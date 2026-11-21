//! Bringing a saved ship back into the universe as it is now: beside the body it was beside,
//! wherever that body has gone, and never inside a planet, a moon or a star.
//!
//! A ship that would come back inside a body is moved along the same direction from the
//! centre of that body up to a circular orbit, with the velocity of that orbit, so it stays up.

use bevy::math::{DMat3, DQuat, DVec3};

use super::body::{Ship, ShipState};
use super::frames::planet_velocity;
use super::moons::with_moons;
use super::warp::Warp;
use super::wells::STAR_LOCK;
use crate::space::Frame;
use crate::universe::{BodyId, PlanetSpec, StarId, StarSpec, Universe};
use crate::world::{Globe, Link};

/// Share of the height of an atmosphere left between its top and the safe orbit.
pub const AIR_CLEAR: f64 = 0.15;

/// Radii from the centre of a body under which no safe orbit is, with or without air.
pub const LOW_ORBIT: f64 = 1.1;

/// Radii from the centre of a star under which a ship is too close to it.
pub const STAR_SAFE: f64 = 3.0;

/// Radii from the centre of a star of the safe orbit: just outside its warp lock.
pub const STAR_ORBIT: f64 = STAR_LOCK * 1.05;

/// Metres the centre of a hull may be under the ground read here and still be on it: a landed
/// hull stands on the ground of a chunk, which differs a little.
const HULL_SUNK: f64 = 5.0;

/// Times the speed of a circular orbit above which a ship in the air of a body, in the frame
/// of its star, did not get there by flying: the body came over it while nobody played.
const TOO_FAST: f64 = 2.0;

/// What the universe is like where and when a ship comes back.
#[derive(Clone, Copy)]
pub struct Sky<'a> {
    /// The universe.
    pub universe: &'a Universe,
    /// Universe time in seconds.
    pub time: f64,
    /// The body the world starts on and its ground, which is then not built again.
    pub land: (BodyId, &'a Globe),
}

/// A body a ship came back inside of, in the frame of its star, and the safe orbit around it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hazard {
    /// The star of the frame.
    pub star: StarId,
    /// Centre of the body in metres.
    pub pos: DVec3,
    /// Velocity of the body in metres per second.
    pub vel: DVec3,
    /// Gravitational parameter of the body in cubic metres per second squared.
    pub mu: f64,
    /// Radius in metres of the safe circular orbit around the centre.
    pub orbit: f64,
    /// Unit direction from the centre to the ship, in the axes of the system.
    pub dir: DVec3,
}

/// What became of a saved ship.
#[derive(Debug, Clone, PartialEq)]
pub enum Restored {
    /// The ship is where the record says, with the seat it was saved with.
    Kept(Ship, bool),
    /// The ship was inside a body with the player seated: it is on a safe orbit around it.
    Orbit(Ship),
    /// The record names a body or a star that does not exist, does not read, or puts an empty
    /// ship inside a body: there is no ship to bring back.
    Lost,
}

/// Returns `state` where the body it is beside is at `time` seconds: the same position and
/// velocity against that body. A record taken at that very time, or beside no body, is
/// returned as it is.
pub fn follow(mut state: ShipState, universe: &Universe, time: f64) -> ShipState {
    let moved = |beside: &super::body::Beside| beside.clock != time && universe.has(beside.body);
    let Some(beside) = state.beside.filter(moved) else {
        return state;
    };
    if state.frame != Frame::System(beside.body.star) {
        return state;
    }
    let planet = universe.planet(beside.body);
    state.pos = (planet.position(time) + DVec3::from(beside.pos)).to_array();
    state.vel = (planet_velocity(&planet, time) + DVec3::from(beside.vel)).to_array();
    state
}

impl Hazard {
    /// Returns the hazard of the planet or moon `planet` at `time` seconds, for a ship in the
    /// direction `dir` from its centre: the orbit is just above its air, and no lower than
    /// [`LOW_ORBIT`] radii, which is where it is on a body without air.
    pub fn body(planet: &PlanetSpec, time: f64, dir: DVec3) -> Self {
        let over_air = planet.radius + planet.atmosphere_height * (1.0 + AIR_CLEAR);
        Self {
            star: planet.id.star,
            pos: planet.position(time),
            vel: planet_velocity(planet, time),
            mu: planet.mu(),
            orbit: over_air.max(planet.radius * LOW_ORBIT),
            dir,
        }
    }

    /// Returns the hazard of `star` for a ship in the direction `dir` from its centre.
    pub fn star(star: &StarSpec, dir: DVec3) -> Self {
        Self {
            star: star.id,
            pos: DVec3::ZERO,
            vel: DVec3::ZERO,
            mu: star.mu(),
            orbit: star.radius * STAR_ORBIT,
            dir,
        }
    }

    /// Returns `ship` on the safe orbit, in the frame of the star: in the same direction from the
    /// centre, flying the way the planets go round, nose ahead and roof away from the body, at
    /// the speed of a circular orbit, engines idle, drive off and gear folded.
    pub fn lift(&self, ship: &Ship) -> Ship {
        let ahead = DVec3::Y.cross(self.dir).try_normalize();
        let ahead = ahead.unwrap_or(DVec3::X);
        let up = self.dir;
        let axes = DMat3::from_cols(up.cross(-ahead), up, -ahead);
        Ship {
            vel: self.vel + ahead * (self.mu / self.orbit).sqrt(),
            spin: DVec3::ZERO,
            throttle: 0.0,
            warp: Warp::Off,
            gear: 0.0,
            stowed: true,
            damage: ship.damage,
            assist: ship.assist,
            ..Ship::at_rest(
                Frame::System(self.star),
                self.pos + up * self.orbit,
                DQuat::from_mat3(&axes).normalize(),
            )
        }
    }
}

/// Returns the hazard a ship in the surface frame of `body` is inside of: under its ground, or
/// under the deck of a body without ground. Nothing for a ship on or over it.
fn under(ship: &Ship, body: BodyId, sky: &Sky) -> Option<Hazard> {
    let planet = sky.universe.planet(body);
    let dir = ship.pos.normalize_or(DVec3::Y);
    let built;
    let globe = match sky.land.0 == body {
        true => sky.land.1,
        false => {
            built = Globe::of(&planet);
            &built
        }
    };
    let ground = match globe.has_ground() {
        true => f64::from(globe.height(dir)),
        false => 0.0,
    };
    let inside = ship.pos.length() < planet.radius + ground - HULL_SUNK;
    let turn = Link::orbiting(&planet, sky.time).turn;
    inside.then(|| Hazard::body(&planet, sky.time, turn * dir))
}

/// Returns the hazard a ship in the frame of `star` is inside of: a planet or a moon whose
/// sea level it is under or whose air it crosses far too fast, or the star when it is closer
/// than [`STAR_SAFE`] radii. Nothing for a ship in open space or on a real orbit.
fn among(ship: &Ship, star: &StarSpec, sky: &Sky) -> Option<Hazard> {
    let system = with_moons(sky.universe, sky.universe.system(star.id));
    let hit = |planet: &&PlanetSpec| {
        let gap = planet.position(sky.time).distance(ship.pos);
        let speed = planet_velocity(planet, sky.time).distance(ship.vel);
        let orbital = (planet.mu() / gap.max(planet.radius)).sqrt();
        let in_air = gap < planet.radius + planet.atmosphere_height;
        gap < planet.radius || (in_air && speed > orbital * TOO_FAST)
    };
    if let Some(planet) = system.planets.iter().find(hit) {
        let dir = (ship.pos - planet.position(sky.time)).normalize_or(DVec3::X);
        return Some(Hazard::body(planet, sky.time, dir));
    }
    let close = ship.pos.length() < star.radius * STAR_SAFE;
    close.then(|| Hazard::star(star, ship.pos.normalize_or(DVec3::X)))
}

/// Returns what becomes of the ship of record `state` under `sky`: brought beside its body,
/// see [`follow`], then checked against the universe. Seated inside a body it goes up to a
/// safe orbit; empty inside a body, or in a frame that does not exist, it is lost. Between the
/// stars there is nothing to be inside of.
pub fn restore(state: ShipState, sky: &Sky) -> Restored {
    let state = follow(state, sky.universe, sky.time);
    let Some(ship) = Ship::from_state(&state) else {
        return Restored::Lost;
    };
    let hazard = match ship.frame {
        Frame::Surface(body) if sky.universe.has(body) => under(&ship, body, sky),
        Frame::System(id) => match sky.universe.star(id) {
            Some(star) => among(&ship, &star, sky),
            None => return Restored::Lost,
        },
        Frame::Surface(_) => return Restored::Lost,
        Frame::Galaxy(_) => None,
    };
    match hazard {
        None => Restored::Kept(ship, state.seated),
        Some(hazard) if state.seated => Restored::Orbit(hazard.lift(&ship)),
        Some(_) => Restored::Lost,
    }
}
