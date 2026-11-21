//! The twin of the grazing cover of `render/cloud_field.wesl`: the cover a line of sight that
//! grazes the shell adds through the depth of the puffs, and the body the shell cuts. Only the
//! shaders draw, so the CPU reads it in the tests alone, which hold it to the shader.

use super::seen::{cut, fbm_seen};
use super::{OCTAVES, Spot, threshold};

/// Mean of the noise of [`OCTAVES`] octaves: half of the sum of their weights.
pub const MEAN: f32 = 0.468_75;

/// Spread of the noise of [`OCTAVES`] octaves, its standard deviation, measured over a million
/// points.
pub const SPREAD: f32 = 0.1065;

/// The share of the noise above `MEAN + z * SPREAD` is `1 / (1 + exp(a z + b z^3))` with these
/// `(a, b)`: a logistic with a cubic, as the noise is bounded and its tails are thinner than a
/// bell's. Fitted to the measured noise within a tenth of the share from -3 to 3 spreads.
pub const TAIL: (f32, f32) = (1.469, 0.1396);

/// Noise above the threshold at which a puff is drawn, where its density times its lone cut is a
/// half. Unit: noise value.
pub const CUT: f32 = 0.066;

/// Height of a puff over its width: a cumulus about as tall as it is wide. A line of sight that
/// grazes the layer crosses its depth over a long run and meets the side of many puffs.
pub const TALL: f32 = 1.0;

/// Least sine of the angle between a line of sight and the shell that the grazing cover reads.
pub const GRAZE: f32 = 0.02;

/// Largest log odds of a clear line of sight the grazing cover works with, so 32-bit floats hold.
pub const CLAMP: f32 = 9.0;

/// Returns the log odds of a clear line of sight `z` spreads under the cut. Twin of `lean`.
fn lean(z: f32) -> f32 {
    TAIL.0 * z + TAIL.1 * z * z * z
}

/// Returns the spreads under the cut whose log odds are `y`, the root of [`lean`]. Twin of `unlean`.
fn unlean(y: f32) -> f32 {
    let half = y / (2.0 * TAIL.1);
    let root = (half * half + (TAIL.0 / (3.0 * TAIL.1)).powi(3)).sqrt();
    (half + root).cbrt() + (half - root).cbrt()
}

/// Returns the noise a line of sight `facing` the shell adds, from 0 to 1 as the sine of its
/// angle with the shell, under `cover`. Through a layer of puffs [`TALL`] as tall as wide, the
/// sight crosses `1 + TALL cot` puffs where a view from overhead crosses one, so the odds that it
/// is clear are raised to that power; the noise is lifted by the spreads that take the share of
/// cloud there. It is 0 seen from overhead and grows smoothly to the limb. Twin of `graze`.
pub fn graze(cover: f32, facing: f32) -> f32 {
    let f = facing.abs().clamp(GRAZE, 1.0);
    let crossed = 1.0 + TALL * (1.0 - f * f).sqrt() / f;
    let z = (threshold(cover) + CUT - MEAN) / SPREAD;
    let y = lean(z).clamp(-CLAMP, CLAMP);
    let clear = -crossed * (1.0 + (-y).exp()).ln();
    let seen = (clear - (1.0 - clear.exp()).max(1.0e-6).ln()).clamp(-CLAMP, CLAMP);
    (unlean(y) - unlean(seen)) * SPREAD
}

/// Returns how much cloud the shell draws at `spot` under `cover` for a view `facing` the shell,
/// the sine of its angle with it, with `footprint` cells per pixel. The shell cuts it at a half,
/// over an edge of a pixel and a half. Twin of the body the shader cuts.
pub fn drawn(spot: Spot, cover: f32, (facing, footprint): (f32, f32)) -> f32 {
    cut(
        fbm_seen(spot, OCTAVES, footprint) + graze(cover, facing),
        cover,
    )
}
