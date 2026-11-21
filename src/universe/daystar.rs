//! The star in the sky of a planet: how far the planet has turned, where its star stands in its own
//! frame, how long its day is, and the path of the star over one day.

use std::f64::consts::TAU;

use bevy::math::{DQuat, DVec3};

use super::spec::PlanetSpec;

/// Moments of one day the path of the star is read at: the star moves 7.5 degrees between two of
/// them, so the highest it stands is known to a hundredth of a degree of sine.
const MOMENTS: usize = 48;

/// Returns the rotation from the axes of a planet to the axes of its system at `time` seconds.
///
/// A planet turns around its own +Y, from west to east, once every `day_seconds`.
pub fn turn(day_seconds: f64, time: f64) -> DQuat {
    DQuat::from_rotation_y(TAU * time / day_seconds)
}

/// Returns the unit direction from the centre of `planet` to its star at universe `time`, in the
/// axes of the planet, which turn with it. The sky, the herds and the home base all read it.
pub fn sun(planet: &PlanetSpec, time: f64) -> DVec3 {
    let toward = -planet.position(time).normalize_or(DVec3::X);
    turn(planet.day_seconds, time).inverse() * toward
}

/// Returns the seconds from one noon to the next on a planet or moon, from its physical spin
/// and its orbit alone: nothing sets the length of a day.
///
/// A body turns the same way round as it goes round its star, so each turn the star has moved
/// on a little and noon comes later: the rate of noons is the rate of turns less the rate of
/// years, `1 / (1 / spin - 1 / year)`. A moon goes round the star with its planet, so its year
/// is its planet's. A planet that tides locked to its star has no noon: the result is infinite.
/// A body that turns slower than its year sees the star go backwards: the result is negative.
pub fn solar_day(planet: &PlanetSpec) -> f64 {
    let year = planet
        .host
        .map_or(planet.orbit_seconds, |host| host.seconds);
    1.0 / (1.0 / planet.day_seconds - 1.0 / year)
}

/// Where the star stands in the frame of a planet at evenly spaced moments of one day, from noon to
/// noon. On a planet locked to its star the day never ends and every moment is the same one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Daystar {
    /// Unit directions from the centre of the planet to its star, in the axes of the planet.
    suns: [DVec3; MOMENTS],
}

impl Default for Daystar {
    /// The day of a planet built without an orbit, which turns freely: the star goes once round
    /// its equator.
    fn default() -> Self {
        let around = |k: usize| TAU * k as f64 / MOMENTS as f64;
        Self {
            suns: std::array::from_fn(|k| DVec3::new(around(k).cos(), 0.0, around(k).sin())),
        }
    }
}

impl Daystar {
    /// Returns the day of `planet`, read from [`sun`] over one [`solar_day`] from time 0.
    pub fn of(planet: &PlanetSpec) -> Self {
        let day = solar_day(planet);
        let step = match day.is_finite() {
            true => day.abs() / MOMENTS as f64,
            false => 0.0,
        };
        Self {
            suns: std::array::from_fn(|k| sun(planet, step * k as f64)),
        }
    }

    /// Returns the sine of the greatest height the star reaches above the horizon over a day at
    /// unit direction `dir` of the planet frame: 1 with the star overhead, 0 on the horizon,
    /// negative where it never rises.
    pub fn peak(&self, dir: DVec3) -> f64 {
        let heights = self.suns.iter().map(|sun| sun.dot(dir));
        heights.fold(f64::MIN, f64::max)
    }
}
