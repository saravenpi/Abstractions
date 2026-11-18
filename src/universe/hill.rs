//! Moon orbits: how many moons fit around a planet and where, inside its Hill sphere.

use super::hash::{self, Rng};
use super::planet::PLANET_RADIUS;
use super::record::PlanetClass;
use super::spec::{PlanetSpec, StarSpec};

/// Smallest and largest radius of a moon in metres. It stays inside `PLANET_RADIUS`, so the
/// ground of a moon streams like the ground of a planet.
pub const MOON_RADIUS: (f64, f64) = (3000.0, 15_000.0);

/// Fewest and most moons of a gas giant.
pub const GIANT_MOONS: (u8, u8) = (2, 6);

/// Radius of the innermost moon orbit allowed, in radii of the planet.
pub const INNERMOST: f64 = 4.0;

/// Share of the Hill radius every moon stays inside: past a third, the star pulls moons away.
pub const HILL_SHARE: f64 = 1.0 / 3.0;

/// Least gap between two neighbouring moon orbits, in sums of the radii of both moons.
pub const SPACING: f64 = 4.0;

/// Child of the planet seed the moon draws come from.
const MOON_STREAM: u64 = 3;

/// Where one moon goes around its planet.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Place {
    /// Radius of the circular orbit in metres from the centre of the planet.
    pub orbit: f64,
    /// Radius of the moon in metres.
    pub radius: f64,
    /// Seed of the moon, its ground and its record.
    pub seed: u64,
}

/// Returns the Hill radius of a planet around its star in metres: `a (m / 3 M)^(1/3)`.
///
/// The planet mass is its gravity times its radius squared over Newton's constant, so the
/// mass ratio is the ratio of the gravitational parameters.
pub(super) fn hill(star: &StarSpec, planet: &PlanetSpec) -> f64 {
    planet.orbit_radius * (planet.mu() / (3.0 * star.mu())).cbrt()
}

/// Returns how many moons a planet asks for before they are fitted around it.
///
/// A gas giant asks for 2 to 6. A solid planet asks for 0 to 2, more often the bigger it is:
/// one in seven of the smallest ask for one, six in ten of the largest.
fn wanted(rng: &mut Rng, planet: &PlanetSpec) -> u32 {
    if planet.record.class == PlanetClass::GasGiant {
        let span = GIANT_MOONS.1 - GIANT_MOONS.0;
        return u32::from(GIANT_MOONS.0) + rng.below(u32::from(span) + 1);
    }
    let size = (planet.radius / PLANET_RADIUS.0).ln() / (PLANET_RADIUS.1 / PLANET_RADIUS.0).ln();
    let chance = 0.15 + 0.45 * size.clamp(0.0, 1.0);
    let roll = rng.unit();
    u32::from(roll < chance) + u32::from(roll < chance * 0.3)
}

/// The orbits the moons of one planet may use: from `low` to `high` metres out, for moons of
/// at most `largest` metres.
struct Span {
    /// Radius of the innermost orbit allowed, in metres.
    low: f64,
    /// Radius in metres no moon reaches past.
    high: f64,
    /// Largest radius of a moon of this planet, in metres.
    largest: f64,
}

/// Returns the span of moon orbits of a planet, or nothing when no moon fits around it.
///
/// Orbits run from 4 planet radii to a third of the Hill radius. Moons are 3 to 15 km, and at
/// most half the radius of a solid planet. A moon has no moons.
fn span(star: &StarSpec, planet: &PlanetSpec) -> Option<Span> {
    let giant = planet.record.class == PlanetClass::GasGiant;
    let half = if giant {
        f64::INFINITY
    } else {
        0.5 * planet.radius
    };
    let span = Span {
        low: INNERMOST * planet.radius,
        high: HILL_SHARE * hill(star, planet),
        largest: MOON_RADIUS.1.min(half),
    };
    let fits = span.largest >= MOON_RADIUS.0 && span.high > span.low;
    (fits && !planet.id.is_moon()).then_some(span)
}

/// Draws moon `index` of `count` in `span`, outside the moon `inside` it.
///
/// The span is cut into one equal band per moon on a log scale and the moon sits in the middle
/// 70 percent of its band, then is pushed out until it clears the moon inside it by
/// [`SPACING`] times their radii.
fn place(rng: &mut Rng, span: &Span, inside: Place, (index, count): (u32, u32)) -> Place {
    let radius = MOON_RADIUS.0 * (span.largest / MOON_RADIUS.0).powf(rng.unit());
    let band = (f64::from(index) + rng.range(0.15, 0.85)) / f64::from(count);
    let clear = inside.orbit + SPACING * (inside.radius + radius);
    Place {
        orbit: (span.low * (span.high / span.low).powf(band)).max(clear),
        radius,
        seed: rng.next(),
    }
}

/// Returns the moons of a planet as places, outward, and how many of the slots are used.
///
/// Each moon the planet asks for is drawn by [`place`] inside the [`span`] of the planet. A
/// moon that does not fit inside the span is dropped with every one beyond it, and the draws
/// go on, so the moons kept never depend on the ones dropped.
pub(super) fn places(star: &StarSpec, planet: &PlanetSpec) -> ([Place; 6], usize) {
    let mut slots = [Place::default(); GIANT_MOONS.1 as usize];
    let mut rng = Rng::new(hash::child(planet.seed, MOON_STREAM));
    let wanted = wanted(&mut rng, planet);
    let Some(span) = span(star, planet) else {
        return (slots, 0);
    };
    let mut used = 0usize;
    for index in 0..wanted {
        let inside = used
            .checked_sub(1)
            .map_or(Place::default(), |last| slots[last]);
        let next = place(&mut rng, &span, inside, (index, wanted));
        if used == index as usize && next.orbit + next.radius <= span.high {
            slots[used] = next;
            used += 1;
        }
    }
    (slots, used)
}
