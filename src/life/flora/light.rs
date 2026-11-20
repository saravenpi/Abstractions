//! How the light of its star and the tint of its species set the colour of a leaf.
//!
//! Leaves are tuned to the light that reaches the ground, after Nancy Y. Kiang, "The Color of Plants on Other
//! Worlds", Scientific American 298 (4), April 2008,
//! <https://www.scientificamerican.com/article/the-color-of-plants-on-other-worlds/>. Under a red dwarf most
//! photons are near infrared, so plants that harvest the whole visible band and beyond reflect little and look
//! nearly black. Under a hot F star the flood of blue photons is harvested and the lower quality green to red
//! light is sent back, so leaves are paler and warmer. Between them, around G and K stars, leaves keep the
//! pigment of their planet with small shifts.

use crate::life::chem::pigment;

/// Star temperature in kelvin over which leaves start to darken towards black.
const DIM: f32 = 3900.0;

/// Star temperature in kelvin at which leaves are as dark as they get.
const DARKEST: f32 = 2600.0;

/// Brightness leaves keep under the coolest stars, against their pigment.
const BLACK: f32 = 0.3;

/// Star temperature in kelvin over which leaves start to send back the green to red light.
const BLUE: f32 = 6500.0;

/// Star temperature in kelvin at which leaves are as pale and warm as they get.
const PALEST: f32 = 10_000.0;

/// Linear RGB a leaf that harvests only blue light moves towards: the green to red it reflects.
const REFLECTED: [f32; 3] = [0.8, 0.52, 0.3];

/// Share of the way to [`REFLECTED`] a leaf goes under the hottest stars.
const PALING: f32 = 0.55;

/// Coolest and hottest star temperature in kelvin the pigment is searched over, and the search step.
const SEARCH: (f32, f32, f32) = (2000.0, 12_000.0, 50.0);

/// Returns the share `x` has moved from `a` to `b`, eased and clamped to 0 to 1.
fn ease(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the temperature in kelvin of the star whose light gave `colour`, the pigment of a planet: the
/// nearest point of the pigment curve. The globe does not keep its star, its life does through the pigment.
pub fn star(colour: [f32; 3]) -> f32 {
    let (low, high, step) = SEARCH;
    let off = |k: f32| {
        let p = pigment(k);
        (0..3).map(|i| (p[i] - colour[i]).powi(2)).sum::<f32>()
    };
    let steps = ((high - low) / step) as u32;
    let all = (0..=steps).map(|i| low + step * i as f32);
    all.min_by(|a, b| off(*a).total_cmp(&off(*b)))
        .unwrap_or(5800.0)
}

/// Returns leaf colour `c` under a star of `kelvin`: darker towards black under a red dwarf, paler and
/// warmer under a hot star, unchanged between.
pub fn under(c: [f32; 3], kelvin: f32) -> [f32; 3] {
    let dark = 1.0 - (1.0 - BLACK) * ease(DIM, DARKEST, kelvin);
    let pale = PALING * ease(BLUE, PALEST, kelvin);
    std::array::from_fn(|i| (c[i] * dark) * (1.0 - pale) + REFLECTED[i] * pale)
}

/// Returns leaf colour `c` shifted by species `tint`, -1 to 1: deeper and cooler below 0, paler and
/// warmer above, by at most a third of its brightness.
pub fn tinted(c: [f32; 3], tint: f32) -> [f32; 3] {
    let t = tint.clamp(-1.0, 1.0);
    let k = 1.0 + 0.33 * t;
    let warm = [1.0 + 0.12 * t, 1.0 + 0.04 * t, 1.0 - 0.15 * t];
    std::array::from_fn(|i| (c[i] * k * warm[i]).clamp(0.0, 1.0))
}
