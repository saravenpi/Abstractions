//! The cloud around the camera as a volume: the same field the shell draws, given a depth by the
//! cloud band, sampled on a lattice fixed to the planet so each puff stays at its place and the
//! ship flies through them at its real speed.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::super::cross::depth_at;
use crate::render::celestial::smoothstep;
use crate::weather::density_at;

/// Metres between two points of the lattice the puffs sit on.
pub const SPACING: f64 = 45.0;

/// Metres around the camera within which puffs are drawn, and the share of it over which they
/// fade out at its edge.
pub const REACH: f64 = 220.0;

/// Most puffs drawn at once.
pub const PUFFS: usize = 256;

/// Volume share under which a point of the lattice holds no puff.
const EMPTY: f32 = 0.02;

/// Metres from the camera under which a puff fades away, so none covers the whole view.
const NEAR: (f32, f32) = (4.0, 20.0);

/// Darkest share of the sunlit cloud colour a puff takes, at the base of the band; it is the
/// full colour at the top.
pub const FLOOR: f32 = 0.78;

/// The cloud layer of a planet as the volume reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer {
    /// Heights in metres above the sea of the base and the top of the cloud band.
    pub band: (f32, f32),
    /// Radius of the planet in metres.
    pub ground: f64,
    /// Noise cells in the radius of the shell the field is drawn on.
    pub cells: f32,
    /// Universe time in seconds, which drifts the field.
    pub seconds: f64,
    /// Broad cover of the sky, from 0 to 1.
    pub cover: f32,
}

/// One puff of the volume.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Puff {
    /// Its centre in planet-fixed axes, in metres from the centre of the planet.
    pub at: DVec3,
    /// How much cloud it holds, from 0 to 1.
    pub fill: f32,
    /// How high in the band it sits, from 0 at the base to 1 at the top.
    pub height: f32,
}

/// Returns how much cloud there is at the planet-fixed unit direction `dir` and `altitude`
/// metres above the sea: the volume of the slices the layer draws over the field there, from the
/// base of the band to the highest slice the column reaches. It is what the camera whites out
/// by, see `CloudCrossing`, so no puff is where no slice is drawn.
pub fn volume(dir: DVec3, altitude: f32, layer: &Layer) -> f32 {
    let body = density_at(dir, layer.cells, layer.seconds, layer.cover);
    depth_at(altitude, layer.band, body)
}

/// Returns a fixed offset in `[-0.3, 0.3)` lattice steps for the point `cell` on each axis, so
/// the puffs do not line up.
fn jitter(cell: IVec3) -> DVec3 {
    let hash = |salt: i32| {
        let mut x = (cell.x.wrapping_mul(73_856_093) ^ cell.y.wrapping_mul(19_349_663))
            ^ cell.z.wrapping_mul(83_492_791)
            ^ salt.wrapping_mul(2_654_435);
        x ^= x >> 13;
        x = x.wrapping_mul(0x5bd1_e995);
        f64::from((x >> 8) & 0xffff) / 65_536.0 - 0.5
    };
    DVec3::new(hash(1), hash(2), hash(3)) * 0.6
}

/// Fills `out` with the puffs within [`REACH`] of the camera at `eye`, planet-fixed, nearest
/// first, at most [`PUFFS`] of them.
pub fn lattice(eye: DVec3, layer: &Layer, out: &mut Vec<Puff>) {
    out.clear();
    let span = (REACH / SPACING).ceil() as i32;
    let side = 2 * span + 1;
    let home = (eye / SPACING).floor().as_ivec3();
    for index in 0..side * side * side {
        let step = IVec3::new(index % side, index / side % side, index / (side * side));
        let cell = home + step - IVec3::splat(span);
        let at = (cell.as_dvec3() + DVec3::splat(0.5) + jitter(cell)) * SPACING;
        if at.distance(eye) <= REACH {
            sample(at, layer, out);
        }
    }
    let near = |puff: &Puff| puff.at.distance_squared(eye);
    out.sort_by(|a, b| near(a).total_cmp(&near(b)));
    out.truncate(PUFFS);
}

/// Adds the puff at the planet-fixed point `at` to `out` when it holds cloud.
fn sample(at: DVec3, layer: &Layer, out: &mut Vec<Puff>) {
    let altitude = (at.length() - layer.ground) as f32;
    let fill = volume(at.normalize(), altitude, layer);
    if fill > EMPTY {
        let height = smoothstep(layer.band.0, layer.band.1, altitude);
        out.push(Puff { at, fill, height });
    }
}

/// Returns the strength a puff `distance` metres from the camera is drawn with for a `fill`:
/// faded near the camera and at the edge of [`REACH`].
pub fn opacity(fill: f32, distance: f32) -> f32 {
    let reach = REACH as f32;
    fill * smoothstep(NEAR.0, NEAR.1, distance) * (1.0 - smoothstep(0.7 * reach, reach, distance))
}

/// Returns the linear colour of a puff `height` up the band, from the sunlit cloud colour
/// `cloud`: a little greyer at the base, never under [`FLOOR`] of it.
pub fn tint(cloud: Vec3, height: f32) -> Vec3 {
    cloud * (FLOOR + (1.0 - FLOOR) * height.clamp(0.0, 1.0))
}
