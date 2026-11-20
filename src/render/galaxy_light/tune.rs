//! Tuning of the bake of the galaxy light.

/// Seconds the last bake must be old before the next one starts, so fast flight through many
/// galaxies bakes a few times, not back to back. Unit: seconds of the app clock.
pub const MIN_GAP: f64 = 4.0;

/// The bake runs on the cores of the machine divided by this: half of them leave the rest to
/// the game. Unit: divisor.
pub const SHARE: usize = 2;

/// Light that one star per sector gives along one light-year of a line of sight, before the
/// dust. Unit: light per (star per sector x light-year).
///
/// Derived from the bake of 0.10.1, whose gain was 1/700 and whose home galaxy read
/// [`REFERENCE`] toward its core from 0.45 radii out. Both models have the same shape in radii,
/// so along that line the column of stars, in stars per sector x light-years, differs by the peak
/// densities and by the radius alone. The line crosses the disc for 0.223 radii, the arm for
/// 0.223 x 0.31, and the core for 1.25 x 0.075. 0.10.1 had peaks of 0.5 (disc), 2 (arm) and 3
/// (core) stars per sector and a radius of 800 to 2,500 game light-years: a colour weighted
/// column of 0.41 radii. The real scale has 22.6, 33.9 and 12, and a radius of 25,000 to 75,000
/// light-years, 30.6 times the old for the same draw: 6.4 radii. The ratio is 6.4 / 0.41 x 30.6,
/// 480. The dust of the real scale is 15 optical depths per radius against 6.6, which
/// takes the mean transmission along that line from 0.68 to 0.40, a factor 1.7 the gain must
/// win back. 1/700 / 480 x 1.7 is 1/198,000, here 1/200,000.
pub const GAIN: f64 = 1.0 / 200_000.0;

/// Light the home galaxy gave toward its core, in the brightest texel of the map, from 0.45
/// radii out along a spoke in its disc, in the bake of 0.10.1. Unit: light before the byte
/// encoding. This is the figure its `GAIN` was set for; the lead replaces it by the number
/// measured on the old tree.
#[cfg(test)]
pub const REFERENCE: f32 = 0.7;

/// Share of [`REFERENCE`] the new bake may differ by at that place.
#[cfg(test)]
pub const REFERENCE_SLACK: f32 = 0.15;

/// Seconds a new map takes to fade in over the one on show, so the sky grows between two bakes
/// and does not change in steps: as long as the minimum gap between bakes. Unit: seconds of the
/// app clock.
pub const FADE: f64 = MIN_GAP;
