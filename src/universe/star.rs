//! Stars: a class drawn from weighted odds biased by the region sets the mass, and the mass sets
//! the light, the size, the temperature and how long the star lives.

use std::f64::consts::SQRT_2;

use bevy::math::{DVec3, Vec3};

use super::density::Region;
use super::hash::{self, Rng};
use super::ids::StarId;
use super::spec::{StarClass, StarSpec};
use super::units::{SUN_MASS, SUN_RADIUS, SUN_TEMPERATURE, YEAR};
use super::{SECTOR_M, names};

/// Years a star of the mass and light of the Sun shines before it leaves the main sequence.
const SUN_LIFE: f64 = 1.0e10;

/// Age in years of the oldest stars: none is older than its galaxy.
pub const OLDEST: f64 = 1.2e10;

/// Youngest a star is found, as a share of the life it has had time for: in the disc and the
/// arms, and among the old stars of the core and the halo.
const YOUNGEST: (f64, f64) = (0.02, 0.5);

/// What a spectral class means: how common it is and how heavy its stars are.
struct Kind {
    /// Spectral class.
    class: StarClass,
    /// Odds in the disc, in an arm, and among the old stars of the core and the halo.
    odds: [u32; 3],
    /// Lightest and heaviest mass, in masses of the Sun.
    mass: (f64, f64),
    /// Linear RGB colour of the light.
    color: Vec3,
}

/// Builds one row of the class table.
const fn kind(class: StarClass, odds: [u32; 3], mass: (f64, f64), color: [f32; 3]) -> Kind {
    Kind {
        class,
        odds,
        mass,
        color: Vec3::new(color[0], color[1], color[2]),
    }
}

/// Every class from hottest to coolest, with the masses of main sequence stars. Hot young stars
/// (O and B) exist only in arms, and the core and the halo hold no star hotter than F. Masses
/// run from 0.08 Suns, the lightest star that burns hydrogen, to 30.
const KINDS: [Kind; 7] = [
    kind(StarClass::O, [0, 4, 0], (16.0, 30.0), [0.6, 0.7, 1.0]),
    kind(StarClass::B, [0, 16, 0], (2.1, 16.0), [0.7, 0.8, 1.0]),
    kind(StarClass::A, [8, 30, 0], (1.4, 2.1), [0.85, 0.9, 1.0]),
    kind(StarClass::F, [32, 50, 20], (1.04, 1.4), [1.0, 1.0, 0.95]),
    kind(StarClass::G, [80, 80, 120], (0.8, 1.04), [1.0, 0.96, 0.88]),
    kind(
        StarClass::K,
        [130, 120, 260],
        (0.45, 0.8),
        [1.0, 0.78, 0.58],
    ),
    kind(
        StarClass::M,
        [750, 700, 600],
        (0.08, 0.45),
        [1.0, 0.55, 0.35],
    ),
];

/// Draws a class from the odds of a region.
fn pick(rng: &mut Rng, region: Region) -> &'static Kind {
    let column = match region {
        Region::Disc => 0,
        Region::Arm => 1,
        Region::Core | Region::Halo => 2,
    };
    let total = KINDS.iter().map(|k| k.odds[column]).sum();
    let mut roll = rng.below(total);
    let hit = |k: &&Kind| {
        let hit = roll < k.odds[column];
        roll = roll.saturating_sub(k.odds[column]);
        hit
    };
    KINDS.iter().find(hit).unwrap_or(&KINDS[6])
}

/// Returns the light of a main sequence star of `mass` Suns, in luminosities of the Sun.
///
/// This is the mass-luminosity relation in three pieces: `0.23 M^2.3` under 0.43 Suns, `M^4` up
/// to 2 and `sqrt(2) M^3.5` above, which joins the piece before. It runs from 0.0007 for the
/// lightest red dwarf to 210,000 for the heaviest blue star.
pub(super) fn light(mass: f64) -> f64 {
    if mass < 0.43 {
        0.23 * mass.powf(2.3)
    } else if mass < 2.0 {
        mass.powi(4)
    } else {
        SQRT_2 * mass.powf(3.5)
    }
}

/// Returns the radius of a main sequence star of `mass` Suns, in radii of the Sun: `M^0.8` up to
/// the mass of the Sun and `M^0.57` above, from 0.13 to 7.
pub(super) fn size(mass: f64) -> f64 {
    mass.powf(if mass < 1.0 { 0.8 } else { 0.57 })
}

/// Returns the years a star of `mass` Suns shines: its fuel over the rate it burns it,
/// `10 billion * M / L`. A red dwarf outlives its galaxy, the heaviest blue star lasts 1.5
/// million years.
pub(super) fn life(mass: f64) -> f64 {
    SUN_LIFE * mass / light(mass)
}

/// Returns the age in years of a star of `mass` Suns in `region` for a draw `roll` from 0 to 1.
///
/// A star is found at an even share of the life it has had time for, the shorter of its own
/// and [`OLDEST`]. The core and the halo stopped making stars long ago, so theirs are past half
/// of it.
fn age(mass: f64, region: Region, roll: f64) -> f64 {
    let young = match region {
        Region::Core | Region::Halo => YOUNGEST.1,
        Region::Disc | Region::Arm => YOUNGEST.0,
    };
    (young + (1.0 - young) * roll) * life(mass).min(OLDEST)
}

/// Builds the star `id` from its own seed and the region of its sector.
///
/// One draw places the mass inside the range of its class. Light and size follow the mass, and
/// the temperature follows both: `5,780 K * (L / R^2)^0.25`, from 2,570 K for the coolest red
/// dwarf to 46,900 K for the hottest blue star.
pub(super) fn build(id: StarId, seed: u64, region: Region) -> StarSpec {
    let mut rng = Rng::new(seed);
    let kind = pick(&mut rng, region);
    let mass = kind.mass.0 + (kind.mass.1 - kind.mass.0) * rng.unit();
    let within = DVec3::new(rng.unit(), rng.unit(), rng.unit());
    let (luminosity, radius) = (light(mass), size(mass));
    StarSpec {
        id,
        seed,
        name: names::star(hash::child(seed, 1)),
        class: kind.class,
        mass: mass * SUN_MASS,
        luminosity,
        radius: radius * SUN_RADIUS,
        temperature: SUN_TEMPERATURE * (luminosity / (radius * radius)).powf(0.25),
        age: age(mass, region, rng.unit()) * YEAR,
        color: kind.color,
        pos: (id.sector.as_dvec3() + within) * SECTOR_M,
        local: within * SECTOR_M,
    }
}
