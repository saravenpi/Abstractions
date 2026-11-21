//! The puffs as the shells draw them: the noise read down to the detail a pixel can hold, and the
//! cut that makes a puff of it.
//!
//! The size of a puff is the field's: its first octave is always read, and only the finer octaves
//! fade once their cells are under a few pixels, with the spread of what is left stretched back
//! to the spread of the whole noise. So a far puff covers as much sky as a near one and keeps its
//! size in metres; the old fade to the mean shrank far puffs to specks, then to nothing. A puff
//! whose first octave is under a pixel is point sampled, which keeps its cover on average.

use bevy::math::DVec3;

use super::{
    EDGE, LONE, OCTAVE_SCALE, OCTAVE_SHIFT, OCTAVES, Spot, drift, space_noise, spot_at, threshold,
};
use crate::render::smoothstep;

/// Cells per pixel of an octave over which it fades out: whole at four pixels a cell, gone at two.
pub const SHARP: (f32, f32) = (0.25, 0.5);

/// Returns `octaves` octaves of the noise at `spot` as seen with `footprint` cells per pixel:
/// the first octave whole, the finer ones faded out once their cells are under two to four
/// pixels, and the spread of the sum brought back to that of all of them. Twin of `fbm_seen`.
pub fn fbm_seen(spot: Spot, octaves: u32, footprint: f32) -> f32 {
    let mut total = 0.5 * (space_noise(spot.base) - 0.5);
    let (mut full, mut kept, mut amplitude, mut scale) = (0.25, 0.25, 0.25, OCTAVE_SCALE);
    let mut q = spot.fine * OCTAVE_SCALE + OCTAVE_SHIFT;
    for _ in 1..octaves {
        let keep = 1.0 - smoothstep(SHARP.0, SHARP.1, footprint * scale);
        if keep > 0.0 {
            total += amplitude * keep * (space_noise(q) - 0.5);
        }
        full += amplitude * amplitude;
        kept += amplitude * amplitude * keep * keep;
        q = q * OCTAVE_SCALE + OCTAVE_SHIFT;
        amplitude *= 0.5;
        scale *= OCTAVE_SCALE;
    }
    0.5 - 0.5f32.powi(octaves as i32 + 1) + total * (full / kept).sqrt()
}

/// Returns how much cloud the noise value `noise` makes under the broad `cover`: the soft step
/// over the threshold times the cut of lone tips, 0 in clear air and 1 inside a puff. Twin of
/// `density` times `lone` in the shader.
pub fn cut(noise: f32, cover: f32) -> f32 {
    let from = threshold(cover);
    let dense = smoothstep(from, from + EDGE, noise);
    let lone = smoothstep(LONE.0, LONE.1, dense);
    dense * (lone + (1.0 - lone) * cover.clamp(0.0, 1.0))
}

/// Returns how much cloud there is at the point `spot` of the shell under the broad `cover`, with
/// all its detail: 0 in clear air, 1 inside a puff. This is what the shadows, the whiteout and
/// the fly-through read, and what the shells draw seen from overhead.
pub fn density(spot: Spot, cover: f32) -> f32 {
    cut(fbm_seen(spot, OCTAVES, 0.0), cover)
}

/// Returns how much cloud there is in the planet-fixed unit direction `dir` of a shell
/// `cells` noise cells in radius, at `seconds` of universe time, under the broad `cover`.
pub fn density_at(dir: DVec3, cells: f32, seconds: f64, cover: f32) -> f32 {
    density(spot_at(dir.as_vec3() * cells, drift(seconds, cells)), cover)
}
