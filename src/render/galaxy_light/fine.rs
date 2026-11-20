//! Sub-texel sampling for the galaxies far outside the camera. One texel of the map spans 0.7
//! degrees at the middle of a face, and a galaxy 2.5 million light-years off spans about 2: a
//! thin disc seen edge on is narrower than a texel, so one line of sight per texel would hit it
//! or miss it by chance and the galaxy would flicker as the camera moves. A texel near such a
//! galaxy is the mean of a grid of lines of sight instead, from the same light model.
//!
//! The grid is finer across the thin side of a disc. The slab of a galaxy 2.5 million
//! light-years off is 60 times thinner than a texel, and a grid that steps over its Gaussian
//! profile lights or darkens the whole galaxy by chance. A texel side that follows the normal of
//! the disc gets two lines of sight per half thickness of the slab, the other side one per 5 percent
//! of the radius.

use bevy::math::{DVec3, Vec3};

use super::march::{Source, radiance};
use crate::weather::face_dir;

/// Most lines of sight along the side of a texel that follows the normal of the disc.
pub const MOST_THIN: usize = 16;

/// Most lines of sight along the other side of a texel.
pub const MOST_WIDE: usize = 8;

/// Share of the angular radius of a galaxy that one line of sight of a texel may stand for: the
/// core and the lanes are 0.03 radii wide.
const GRAIN: f64 = 0.05;

/// Lines of sight per half thickness of the slab of a galaxy across its disc.
const PER_SLAB: f64 = 2.0;

/// The patch of sky a far galaxy covers and how finely a texel in it is sampled.
#[derive(Debug, Clone, Copy)]
pub(super) struct Cone {
    /// Unit vector toward the centre of the galaxy.
    axis: DVec3,
    /// Cosine of the angular radius of the galaxy, a texel wider.
    cos: f64,
    /// Lines of sight along a texel side in the direction of the normal of the disc.
    thin: usize,
    /// Lines of sight along a texel side across it.
    wide: usize,
}

/// Returns the cones of the galaxies in `sources` that a map `side` texels wide cannot resolve
/// with one line of sight per texel: those whose camera lies outside their light, and whose
/// angular radius is under ten texels or whose slab is under one texel thick.
pub(super) fn cones(sources: &[Source], side: usize) -> Vec<Cone> {
    let texel = std::f64::consts::FRAC_PI_2 / side as f64;
    let cone = |source: &Source| {
        let (distance, reach) = (source.offset.length(), source.glow.reach());
        if source.inside || distance <= reach {
            return None;
        }
        let radius = (reach / distance).asin();
        let wide = ((texel / (GRAIN * radius)).ceil() as usize).clamp(1, MOST_WIDE);
        let thin = (texel * PER_SLAB * distance / source.glow.slab()).ceil() as usize;
        let thin = thin.clamp(wide, MOST_THIN);
        (thin > 1).then(|| Cone {
            axis: source.offset / distance,
            cos: (radius + texel).cos(),
            thin,
            wide,
        })
    };
    sources.iter().filter_map(cone).collect()
}

/// Returns the lines of sight along the column and the row sides of a texel in direction `dir`:
/// the finest of the cones that hold it, or one each. `lean` is how far each side runs along the
/// normal of the disc, from 0 to 1.
fn fineness(cones: &[Cone], dir: DVec3, lean: (f64, f64)) -> (usize, usize) {
    let holds = |cone: &&Cone| dir.dot(cone.axis) >= cone.cos;
    let along = |lean: f64, cone: &Cone| {
        let thin = (cone.thin as f64 * lean).ceil() as usize;
        thin.max(cone.wide)
    };
    cones.iter().filter(holds).fold((1, 1), |most, cone| {
        (
            most.0.max(along(lean.0, cone)),
            most.1.max(along(lean.1, cone)),
        )
    })
}

/// Returns the light of texel `column`, `line` of face `face` of a map `side` texels wide: the
/// mean over a grid of lines of sight where a far galaxy needs it, one through the middle of the
/// texel everywhere else.
pub(super) fn texel(
    sources: &[Source],
    cones: &[Cone],
    side: usize,
    at: (usize, usize, usize),
) -> Vec3 {
    let (face, column, line) = at;
    let across = |index: f64| 2.0 * (index + 0.5) / side as f64 - 1.0;
    let dir = |i: f64, j: f64| face_dir(face, across(i), across(j));
    let (c, l) = (column as f64, line as f64);
    let middle = dir(c, l);
    let lean = |a: DVec3, b: DVec3| (a - b).normalize_or_zero().y.abs();
    let lean = (
        lean(dir(c + 0.5, l), dir(c - 0.5, l)),
        lean(dir(c, l + 0.5), dir(c, l - 0.5)),
    );
    let (nx, ny) = fineness(cones, middle, lean);
    if nx * ny == 1 {
        return radiance(sources, middle);
    }
    let sub = |first: f64, k: usize, n: usize| first + (k as f64 + 0.5) / n as f64 - 0.5;
    let mut sum = Vec3::ZERO;
    for j in 0..ny {
        for i in 0..nx {
            sum += radiance(sources, dir(sub(c, i, nx), sub(l, j, ny)));
        }
    }
    sum / (nx * ny) as f32
}
