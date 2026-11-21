//! The planet the ship may cruise over and how the frame of the ship sits against it: in its
//! surface frame the planet itself, in the frame of its star the planet whose lock sphere holds
//! the ship.

use bevy::math::{DQuat, DVec3};

use super::skim::Band;
use crate::ship::body::Ship;
use crate::ship::frames::spin_rate;
use crate::ship::sky::Skies;
use crate::ship::warp::Warp;
use crate::ship::wells::PLANET_LOCK;
use crate::space::Frame;
use crate::universe::{PlanetSpec, turn};

/// The planet under the ship, in the frame of the ship.
#[derive(Debug, Clone, PartialEq)]
pub struct Over {
    /// What the universe says about the planet.
    pub planet: PlanetSpec,
    /// Its centre in metres.
    pub centre: DVec3,
    /// Its velocity in metres per second.
    pub vel: DVec3,
    /// Turn from the axes of the planet to the axes of the frame.
    pub turn: DQuat,
    /// Angular velocity of its ground in radians per second in the axes of the frame: nothing in
    /// its surface frame, which turns with it.
    pub spin: DVec3,
}

impl Over {
    /// Returns the velocity of the ground under the point `rel` metres from the centre, in the
    /// terms of the frame.
    pub fn ground(&self, rel: DVec3) -> DVec3 {
        self.spin.cross(rel)
    }

    /// Returns the unit direction from the centre of the point `at` given in planet axes.
    pub fn toward(&self, at: DVec3) -> DVec3 {
        (self.turn * at).normalize_or(DVec3::Y)
    }
}

/// Returns the planet of the system kept by `skies` whose lock sphere holds `pos` in the frame
/// of the star, the nearest in its own radii, with its index among the wells.
fn locked<'a>(skies: &'a Skies, pos: DVec3) -> Option<(&'a PlanetSpec, usize)> {
    let system = skies.spec()?;
    let reach = |(index, planet): (usize, &'a PlanetSpec)| {
        let well = skies.wells.get(index + 1)?;
        let radii = well.pos.distance(pos) / planet.radius;
        (radii < PLANET_LOCK).then_some((planet, index + 1, radii))
    };
    let found = system.planets.iter().enumerate().filter_map(reach);
    let (planet, index, _) = found.min_by(|a, b| a.2.total_cmp(&b.2))?;
    Some((planet, index))
}

/// Returns the planet under `ship` at universe second `time`, the wells having been sampled
/// `lag` seconds before, or nothing between the stars or far from every planet.
pub fn over(ship: &Ship, skies: &Skies, (time, lag): (f64, f64)) -> Option<Over> {
    match ship.frame {
        Frame::Surface(body) => {
            let planet = skies.spec()?.planets.iter().find(|p| p.id == body)?;
            Some(Over {
                planet: planet.clone(),
                centre: DVec3::ZERO,
                vel: DVec3::ZERO,
                turn: DQuat::IDENTITY,
                spin: DVec3::ZERO,
            })
        }
        Frame::System(_) => {
            let (planet, index) = locked(skies, ship.pos)?;
            let well = skies.wells[index];
            let turn = turn(planet.day_seconds, time);
            Some(Over {
                planet: planet.clone(),
                centre: well.pos + well.vel * lag,
                vel: well.vel,
                turn,
                spin: turn * DVec3::Y * spin_rate(planet),
            })
        }
        Frame::Galaxy(_) => None,
    }
}

/// Returns whether `ship` may start a cruise over `over`: in the frame of the star, at the floor
/// of the cruise or above it, off the ground and with the warp drive off. A climb out of the air
/// goes on to the floor before the cruise takes over.
pub fn free(ship: &Ship, over: &Over) -> bool {
    let height = ship.pos.distance(over.centre) - over.planet.radius;
    let space = matches!(ship.frame, Frame::System(_));
    let floor = Band::of(&over.planet).floor - 1.0;
    space && ship.warp == Warp::Off && !ship.landed && height >= floor
}
