//! Moon orbits: which moons a planet has, by how they form, and where they fit around it.

use std::f64::consts::PI;

use super::hash::{self, Rng};
use super::record::PlanetClass;
use super::rings::ROCHE;
use super::spec::{GRAVITATION, PlanetSpec, StarSpec};

/// Share of the solid planets a giant impact left with one large moon, as the Earth was.
pub const IMPACT_SHARE: f64 = 0.25;

/// Share of the solid planets that caught one or two small moons, as Mars did.
pub const CAPTURE_SHARE: f64 = 0.15;

/// Smallest and largest radius of an impact moon, in radii of its planet.
pub const IMPACT_RADIUS: (f64, f64) = (0.1, 0.3);

/// Closest and farthest orbit of an impact moon, in radii of its planet.
pub const IMPACT_ORBIT: (f64, f64) = (20.0, 80.0);

/// Smallest and largest radius of a captured moon in metres.
pub const CAPTURED_RADIUS: (f64, f64) = (1.0e4, 5.0e4);

/// Closest orbit of a captured moon, as a share of the Hill radius of its planet.
pub const CAPTURED_ORBIT: f64 = 0.15;

/// Fewest and most regular moons of a gas giant, the ones that formed in the disc around it.
pub const REGULAR_MOONS: (u8, u8) = (2, 6);

/// Most small moons a gas giant caught on top of its regular ones.
pub const STRAYS: u8 = 2;

/// Smallest and largest radius of a regular moon in metres: from under Europa to Ganymede.
pub const REGULAR_RADIUS: (f64, f64) = (1.0e6, 2.6e6);

/// Closest and farthest orbit of a regular moon, in radii of its planet.
pub const REGULAR_ORBIT: (f64, f64) = (5.0, 30.0);

/// Most moons one planet holds.
pub const MOST: usize = 6;

/// Share of the Hill radius every moon stays inside: past a half, the star pulls moons away.
pub const HILL_SHARE: f64 = 0.5;

/// Least ratio between the orbits of two neighbouring moons.
pub const SPACING: f64 = 1.2;

/// Lightest and densest matter of an impact moon, a captured one and a regular one, in
/// kilograms per cubic metre: mantle rock, rubble, and ice to rock.
const MATTER: [(f64, f64); 3] = [(3000.0, 3400.0), (1500.0, 2500.0), (1500.0, 3500.0)];

/// Child of the planet seed the moon draws come from.
const MOON_STREAM: u64 = 3;

/// Where one moon goes around its planet, and what it is.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Place {
    /// Radius of the circular orbit in metres from the centre of the planet.
    pub orbit: f64,
    /// Radius of the moon in metres.
    pub radius: f64,
    /// Density of its matter in kilograms per cubic metre, before its own weight squeezes it.
    pub matter: f64,
    /// How hard its neighbours keep its orbit out of round, which is what lets tides heat it:
    /// the square of its eccentricity over that of Io, 0 to 4 for a regular moon, else 0.
    pub forcing: f64,
    /// Seed of the moon, its ground and its record.
    pub seed: u64,
}

/// Returns the Hill radius in metres of a body of gravitational parameter `body_mu` on a circular
/// orbit of `orbit` metres around a mass of parameter `central_mu`: `a (m / 3 M)^(1/3)`.
///
/// This is the one formula every Hill radius in the game comes from. The mass ratio is the
/// ratio of the gravitational parameters.
pub(super) fn radius(body_mu: f64, central_mu: f64, orbit: f64) -> f64 {
    orbit * (body_mu / (3.0 * central_mu)).cbrt()
}

/// Returns the Hill radius of a planet around its star in metres, by [`radius`].
pub(super) fn hill(star: &StarSpec, planet: &PlanetSpec) -> f64 {
    radius(planet.mu(), star.mu(), planet.orbit_radius)
}

/// The moons that fit around one planet so far.
struct Fit {
    /// The moons kept, outward.
    slots: [Place; MOST],
    /// How many of the slots are used.
    used: usize,
    /// Radius in metres every orbit stays outside: the Roche limit for a moon as dense as the
    /// planet.
    low: f64,
    /// Mean density of the planet in kilograms per cubic metre.
    density: f64,
    /// Radius in metres every orbit stays inside: [`HILL_SHARE`] of the Hill radius.
    high: f64,
}

impl Fit {
    /// Keeps `place` when its orbit lies outside the Roche limit, inside the Hill share and
    /// [`SPACING`] times past the moon before it, and drops it otherwise. The Roche limit of a
    /// moon lighter than its planet lies further out, by the cube root of the ratio of their
    /// densities.
    fn add(&mut self, place: Place) {
        let inside = self
            .used
            .checked_sub(1)
            .map_or(0.0, |i| self.slots[i].orbit);
        let roche = self.low * (self.density / place.matter).cbrt().max(1.0);
        let clear = place.orbit > roche.max(inside * SPACING) && place.orbit < self.high;
        if clear && self.used < MOST {
            self.slots[self.used] = place;
            self.used += 1;
        }
    }
}

/// Draws a moon of kind `kind` (0 impact, 1 captured, 2 regular) with a radius spread on a log
/// scale over `radius` metres, on the orbit `orbit` metres out. Every kind takes the same draws.
fn moon(rng: &mut Rng, kind: usize, radius: (f64, f64), orbit: f64) -> Place {
    let forcing = (2.0 * rng.unit()).powi(2);
    Place {
        orbit,
        radius: radius.0 * (radius.1 / radius.0).powf(rng.unit()),
        matter: rng.range(MATTER[kind].0, MATTER[kind].1),
        forcing: if kind == 2 { forcing } else { 0.0 },
        seed: rng.next(),
    }
}

/// Returns the orbit of moon `index` of `count` in the span from `low` to `high` metres: the
/// span is cut into one equal band per moon on a log scale and the moon sits in the middle 70
/// percent of its band.
fn banded(rng: &mut Rng, (low, high): (f64, f64), (index, count): (u32, u32)) -> f64 {
    let band = (f64::from(index) + rng.range(0.15, 0.85)) / f64::from(count);
    low * (high / low).max(1.0).powf(band)
}

/// Adds the moons of a gas giant of `radius` metres to `fit`: 2 to 6 regular moons from 5 to 30
/// radii out, then up to 2 small captured ones from 15 percent of the Hill radius out.
fn around_giant(rng: &mut Rng, fit: &mut Fit, radius: f64) {
    let more = u32::from(REGULAR_MOONS.1 - REGULAR_MOONS.0) + 1;
    let regular = u32::from(REGULAR_MOONS.0) + rng.below(more);
    let span = (REGULAR_ORBIT.0 * radius, REGULAR_ORBIT.1 * radius);
    for index in 0..regular {
        let orbit = banded(rng, span, (index, regular));
        fit.add(moon(rng, 2, REGULAR_RADIUS, orbit));
    }
    let strays = rng.below(u32::from(STRAYS) + 1);
    let far = (CAPTURED_ORBIT / HILL_SHARE * fit.high, fit.high);
    for index in 0..strays {
        let orbit = banded(rng, far, (index, strays));
        fit.add(moon(rng, 1, CAPTURED_RADIUS, orbit));
    }
}

/// Adds the moons of a solid planet of `radius` metres to `fit`: one large impact moon for a
/// quarter of them, 0.1 to 0.3 of the planet across and 20 to 80 radii out, one or two small
/// captured moons for 15 in 100, and none for the rest.
fn around_solid(rng: &mut Rng, fit: &mut Fit, radius: f64) {
    let roll = rng.unit();
    if roll < IMPACT_SHARE {
        let size = (IMPACT_RADIUS.0 * radius, IMPACT_RADIUS.1 * radius);
        let orbit = radius * rng.range(IMPACT_ORBIT.0, IMPACT_ORBIT.1);
        fit.add(moon(rng, 0, size, orbit));
    } else if roll < IMPACT_SHARE + CAPTURE_SHARE {
        let caught = 1 + rng.below(2);
        let far = (CAPTURED_ORBIT / HILL_SHARE * fit.high, fit.high);
        for index in 0..caught {
            let orbit = banded(rng, far, (index, caught));
            fit.add(moon(rng, 1, CAPTURED_RADIUS, orbit));
        }
    }
}

/// Returns the moons of a planet as places, outward, and how many of the slots are used.
///
/// Every orbit lies between the Roche limit and half the Hill radius of the planet. A moon
/// that does not fit there is dropped, and the draws go on, so the moons kept never depend on
/// the ones dropped: a planet close to its star, whose Hill sphere is small, keeps none. A
/// moon has no moons.
pub(super) fn places(star: &StarSpec, planet: &PlanetSpec) -> ([Place; MOST], usize) {
    let mut fit = Fit {
        slots: [Place::default(); MOST],
        used: 0,
        low: ROCHE * planet.radius,
        density: 3.0 * planet.gravity / (4.0 * PI * GRAVITATION * planet.radius),
        high: HILL_SHARE * hill(star, planet),
    };
    let mut rng = Rng::new(hash::child(planet.seed, MOON_STREAM));
    match planet.record.class {
        _ if planet.id.is_moon() => {}
        PlanetClass::GasGiant => around_giant(&mut rng, &mut fit, planet.radius),
        _ => around_solid(&mut rng, &mut fit, planet.radius),
    }
    (fit.slots, fit.used)
}
