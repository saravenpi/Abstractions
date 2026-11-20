//! Marching starlight along lines of sight: the light of every galaxy in view, integrated through
//! the star density of that galaxy with its dust absorbing, into the six faces of a cube map.

use std::thread;

use bevy::math::{DVec3, Vec3};

use super::pack::encode;
use super::tune::SHARE;
use crate::render::smoothstep;
use crate::universe::{GLY, Glow, Parts};
use crate::weather::face_dir;

/// Texels along one side of a face of the map.
pub const SIDE: usize = 128;

/// Faces of the map, in the order of a cube texture.
pub const FACES: usize = 6;

/// Bytes of a map: four per texel.
pub const BYTES: usize = FACES * SIDE * SIDE * 4;

/// Game light-years within which stars are drawn one by one: their light is left out of the
/// integral so nothing counts twice. `SIGHT` in `starfield.wesl` is the same.
pub const NEAR: f32 = 40.0;

/// Game light-years over which the stars drawn one by one fade out, as `SIGHT_FADE` in
/// `starfield.wesl` has them: the integral takes the light they give up over the same stretch.
const FADE: f32 = 2.5;

/// Longest step along a line of sight, as a share of the radius of the galaxy it crosses. The
/// disc and the core have a deviation of 0.02 to 0.1 radii, so with this step no structure falls
/// between two samples, and the light changes without a jump as the camera moves.
const STEP: f64 = 0.03;

/// Fewest and most steps along the part of a line of sight that lies in one galaxy: a short
/// chord through the slab needs few, one along the disc is cut at the most.
const STEPS: (usize, usize) = (6, 96);

/// Light that one star per sector gives along one game light-year of a line of sight: the
/// scale at which the home galaxy reads about 0.7 toward its core from half way out.
const GAIN: f64 = 1.0 / 700.0;

/// Colour of old stars, the core and the halo.
const WARM: Vec3 = Vec3::new(1.0, 0.78, 0.5);

/// Colour of young stars, the arms.
const COOL: Vec3 = Vec3::new(0.62, 0.76, 1.0);

/// Colour of the stars of the plain disc.
const PLAIN: Vec3 = Vec3::new(0.72, 0.7, 0.76);

/// A galaxy seen from the camera.
#[derive(Debug, Clone, Copy)]
pub struct Source {
    /// Its star density and dust.
    pub glow: Glow,
    /// Metres from the camera to its centre, in galaxy axes.
    pub offset: DVec3,
    /// Whether the camera is inside its disc, so that its dust veils every galaxy beyond.
    pub inside: bool,
}

/// Returns the colour of the stars of a place: its density times the colour of each structure.
fn tint(parts: Parts) -> Vec3 {
    WARM * (parts.core + parts.halo) as f32 + COOL * parts.arm as f32 + PLAIN * parts.disc as f32
}

impl Source {
    /// Returns the distances in metres along the line of sight `dir` between which it lies in
    /// the slab and the sphere that hold the light of the galaxy, or nothing when it misses.
    fn span(&self, dir: DVec3) -> Option<(f64, f64)> {
        let (offset, reach) = (self.offset, self.glow.reach());
        let along = dir.dot(offset);
        let gap = along * along - (offset.length_squared() - reach * reach);
        if gap < 0.0 {
            return None;
        }
        let (mut from, mut to) = ((along - gap.sqrt()).max(0.0), along + gap.sqrt());
        let slab = self.glow.slab();
        if dir.y.abs() > 1.0e-9 {
            let (a, b) = ((offset.y - slab) / dir.y, (offset.y + slab) / dir.y);
            (from, to) = (from.max(a.min(b)), to.min(a.max(b)));
        } else if offset.y.abs() > slab {
            return None;
        }
        (to > from).then_some((from, to))
    }

    /// Returns the light that reaches the camera along `dir` from this galaxy, and the optical
    /// depth of its dust on the way. The stars within [`NEAR`] light-years of the camera give no light.
    fn march(&self, dir: DVec3) -> (Vec3, f64) {
        let Some((from, to)) = self.span(dir) else {
            return (Vec3::ZERO, 0.0);
        };
        let wanted = ((to - from) / (STEP * self.glow.radius())).ceil() as usize;
        let steps = wanted.clamp(STEPS.0, STEPS.1);
        let step = (to - from) / steps as f64;
        let (mut light, mut depth) = (Vec3::ZERO, 0.0);
        for k in 0..steps {
            let along = from + step * (k as f64 + 0.5);
            let (stars, dust) = self.glow.at(dir * along - self.offset);
            let dim = dust * step / GLY;
            let far = smoothstep(NEAR - FADE, NEAR, (along / GLY) as f32);
            if far > 0.0 {
                let clear = (-(depth + 0.5 * dim)).exp();
                light += tint(stars) * (far * (clear * GAIN * step / GLY) as f32);
            }
            depth += dim;
        }
        (light, depth)
    }
}

/// Returns the light that reaches the camera along the unit direction `dir` from `sources`:
/// the sum of each, those beyond the galaxy the camera is in dimmed by its dust.
pub fn radiance(sources: &[Source], dir: DVec3) -> Vec3 {
    let (mut light, mut depth) = (Vec3::ZERO, 0.0);
    for own in sources.iter().filter(|source| source.inside) {
        let (seen, dust) = own.march(dir);
        (light, depth) = (light + seen, depth + dust);
    }
    let veil = (-depth).exp() as f32;
    let beyond = sources.iter().filter(|source| !source.inside);
    beyond.fold(light, |sum, source| sum + source.march(dir).0 * veil)
}

/// Returns the texels of rows `first`, `first + stride`, and so on, of a map `side` texels
/// wide, six faces of `side` rows each, as `(row, bytes)` pairs.
pub(super) fn rows(
    sources: &[Source],
    side: usize,
    (first, stride): (usize, usize),
) -> Vec<(usize, Vec<u8>)> {
    let at = |index: usize| 2.0 * (index as f64 + 0.5) / side as f64 - 1.0;
    (first..FACES * side)
        .step_by(stride)
        .map(|row| {
            let (face, line) = (row / side, row % side);
            let texels = (0..side).flat_map(|column| {
                let dir = face_dir(face, at(column), at(line));
                encode(radiance(sources, dir))
            });
            (row, texels.collect())
        })
        .collect()
}

/// Returns the threads a bake runs on: a share of the cores of the machine, see [`SHARE`], up to
/// one per face and at least one.
pub(super) fn workers() -> usize {
    (thread::available_parallelism().map_or(1, usize::from) / SHARE).clamp(1, FACES)
}

/// Bakes the cube map of `sources`, `side` texels along a face, on a share of the cores of the
/// machine, see [`SHARE`], up to one thread per face and at least one. Returns the bytes of the
/// six faces one after the other.
pub fn bake_at(sources: &[Source], side: usize) -> Vec<u8> {
    let workers = workers();
    let mut bytes = vec![0; FACES * side * side * 4];
    thread::scope(|scope| {
        let work: Vec<_> = (0..workers)
            .map(|first| scope.spawn(move || rows(sources, side, (first, workers))))
            .collect();
        for (row, texels) in work
            .into_iter()
            .flat_map(|done| done.join().unwrap_or_default())
        {
            bytes[row * side * 4..][..side * 4].copy_from_slice(&texels);
        }
    });
    bytes
}

/// Bakes the cube map the sky reads: [`SIDE`] texels along a face.
pub fn bake(sources: &[Source]) -> Vec<u8> {
    bake_at(sources, SIDE)
}
