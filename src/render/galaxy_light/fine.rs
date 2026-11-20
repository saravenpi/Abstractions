//! Sub-texel sampling for the galaxies far outside the camera. One texel of the map spans 0.7
//! degrees at the middle of a face, and a galaxy 2.5 million light-years off spans about 2: a
//! thin disc seen edge on is narrower than a texel, so one line of sight per texel would hit it
//! or miss it by chance and the galaxy would flicker as the camera moves. A texel near such a
//! galaxy is the mean of a grid of lines of sight instead, from the same light model.

use bevy::math::{DVec3, Vec3};

use super::march::{Source, radiance};
use crate::weather::face_dir;

/// Most lines of sight along one side of a texel: 36 in all.
pub const MOST: usize = 6;

/// Share of the angular radius of a galaxy that one line of sight of a texel may stand for.
const GRAIN: f64 = 0.1;

/// The patch of sky a far galaxy covers and how finely a texel in it is sampled.
#[derive(Debug, Clone, Copy)]
pub(super) struct Cone {
    /// Unit vector toward the centre of the galaxy.
    axis: DVec3,
    /// Cosine of the angular radius of the galaxy, a texel wider.
    cos: f64,
    /// Lines of sight along one side of a texel in it.
    fine: usize,
}

/// Returns the cones of the galaxies in `sources` that cover so little of the sky that a map
/// `side` texels wide cannot resolve them: those whose camera lies outside their light, with an
/// angular radius under ten texels. Other galaxies are wide enough for one line per texel.
pub(super) fn cones(sources: &[Source], side: usize) -> Vec<Cone> {
    let texel = std::f64::consts::FRAC_PI_2 / side as f64;
    let cone = |source: &Source| {
        let (distance, reach) = (source.offset.length(), source.glow.reach());
        if source.inside || distance <= reach {
            return None;
        }
        let radius = (reach / distance).asin();
        let fine = ((texel / (GRAIN * radius)).ceil() as usize).min(MOST);
        (fine > 1).then(|| Cone {
            axis: source.offset / distance,
            cos: (radius + texel).cos(),
            fine,
        })
    };
    sources.iter().filter_map(cone).collect()
}

/// Returns the lines of sight along one side of a texel in direction `dir`: the finest of the
/// cones that hold it, or one.
fn fineness(cones: &[Cone], dir: DVec3) -> usize {
    let holds = |cone: &&Cone| dir.dot(cone.axis) >= cone.cos;
    cones
        .iter()
        .filter(holds)
        .map(|cone| cone.fine)
        .max()
        .unwrap_or(1)
}

/// Returns the light of texel `column`, `line` of face `face` of a map `side` texels wide: the
/// mean over a grid of lines of sight where a far galaxy needs it, one through the middle of the
/// texel everywhere else.
pub(super) fn texel(sources: &[Source], cones: &[Cone], side: usize, at: (usize, usize, usize)) -> Vec3 {
    let (face, column, line) = at;
    let across = |index: f64| 2.0 * (index + 0.5) / side as f64 - 1.0;
    let middle = face_dir(face, across(column as f64), across(line as f64));
    let n = fineness(cones, middle);
    if n == 1 {
        return radiance(sources, middle);
    }
    let sub = |first: usize, k: usize| first as f64 + (k as f64 + 0.5) / n as f64 - 0.5;
    let mut sum = Vec3::ZERO;
    for j in 0..n {
        for i in 0..n {
            let dir = face_dir(face, across(sub(column, i)), across(sub(line, j)));
            sum += radiance(sources, dir);
        }
    }
    sum / (n * n) as f32
}
