//! Stars: a class drawn from weighted odds biased by the region, which sets mass, light, size and colour.

use bevy::math::{DVec3, Vec3};

use super::density::Region;
use super::hash::{self, Rng};
use super::home::{HOME_STAR_MASS, HOME_STAR_RADIUS, HOME_STAR_TEMPERATURE};
use super::ids::StarId;
use super::spec::{StarClass, StarSpec};
use super::{GLY, SECTOR_GLY, names};

/// What a spectral class means: how common it is and the ranges its stars fall in.
struct Kind {
    /// Spectral class.
    class: StarClass,
    /// Odds in the disc, in an arm, and among the old stars of the core and the halo.
    odds: [u32; 3],
    /// Lightest and heaviest mass, in masses of the home star.
    mass: (f64, f64),
    /// Dimmest and brightest light output, the home star being 1.
    light: (f64, f64),
    /// Smallest and largest radius, in radii of the home star.
    size: (f64, f64),
    /// Linear RGB colour of the light.
    color: Vec3,
}

/// Builds one row of the class table.
const fn kind(class: StarClass, odds: [u32; 3], ranges: [(f64, f64); 3], color: [f32; 3]) -> Kind {
    Kind {
        class,
        odds,
        mass: ranges[0],
        light: ranges[1],
        size: ranges[2],
        color: Vec3::new(color[0], color[1], color[2]),
    }
}

/// Every class from hottest to coolest. Hot young stars (O and B) exist only in arms, and the
/// core and the halo hold no star hotter than F. Masses run from 0.2 to 20 home star masses,
/// light from 0.05 to 50 and radii from 0.3 to 8 home star radii.
const KINDS: [Kind; 7] = [
    kind(
        StarClass::O,
        [0, 4, 0],
        [(8.0, 20.0), (20.0, 50.0), (4.0, 8.0)],
        [0.6, 0.7, 1.0],
    ),
    kind(
        StarClass::B,
        [0, 16, 0],
        [(2.5, 8.0), (8.0, 20.0), (2.0, 4.0)],
        [0.7, 0.8, 1.0],
    ),
    kind(
        StarClass::A,
        [8, 30, 0],
        [(1.5, 2.5), (3.0, 8.0), (1.4, 2.0)],
        [0.85, 0.9, 1.0],
    ),
    kind(
        StarClass::F,
        [32, 50, 20],
        [(1.1, 1.5), (1.4, 3.0), (1.1, 1.4)],
        [1.0, 1.0, 0.95],
    ),
    kind(
        StarClass::G,
        [80, 80, 120],
        [(0.8, 1.1), (0.6, 1.4), (0.9, 1.1)],
        [1.0, 0.96, 0.88],
    ),
    kind(
        StarClass::K,
        [130, 120, 260],
        [(0.5, 0.8), (0.2, 0.6), (0.6, 0.9)],
        [1.0, 0.78, 0.58],
    ),
    kind(
        StarClass::M,
        [750, 700, 600],
        [(0.2, 0.5), (0.05, 0.2), (0.3, 0.6)],
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

/// Returns the surface temperature in kelvin of a star of `light` home luminosities and `size` home radii.
///
/// A surface radiates as the fourth power of its temperature, so the temperature follows the
/// light per unit of surface: the home star, 1 and 1, is at exactly 5,780 K.
pub(super) fn temperature(light: f64, size: f64) -> f64 {
    HOME_STAR_TEMPERATURE * (light / (size * size)).powf(0.25)
}

/// Builds the star `id` from its own seed and the region of its sector.
pub(super) fn build(id: StarId, seed: u64, region: Region) -> StarSpec {
    let mut rng = Rng::new(seed);
    let kind = pick(&mut rng, region);
    let heavy = rng.unit();
    let within = DVec3::new(rng.unit(), rng.unit(), rng.unit());
    let blend = |span: (f64, f64)| span.0 + (span.1 - span.0) * heavy;
    StarSpec {
        id,
        seed,
        name: names::star(hash::child(seed, 1)),
        class: kind.class,
        mass: blend(kind.mass) * HOME_STAR_MASS,
        luminosity: blend(kind.light),
        radius: blend(kind.size) * HOME_STAR_RADIUS,
        temperature: temperature(blend(kind.light), blend(kind.size)),
        color: kind.color,
        pos: (id.sector.as_dvec3() + within) * SECTOR_GLY * GLY,
    }
}
