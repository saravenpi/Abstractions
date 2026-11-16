//! Relief above the coast: rolling hills, terraced plateaus, foothills and ridged mountains.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin, RidgedMulti};

use super::{smooth, sub_seed};

/// Snaps `v` to steps of `step` with steep risers, giving terraced cliffs.
fn terrace(v: f32, step: f32) -> f32 {
    let t = v / step;
    let k = t.floor();
    (k + smooth(0.8, 1.0, t - k)) * step
}

/// The noise fields of the inland relief.
#[derive(Clone)]
pub struct Relief {
    mount: Fbm<Perlin>,
    ridge: RidgedMulti<Perlin>,
    hills: Fbm<Perlin>,
}

impl Relief {
    /// Builds the relief fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self {
            mount: Fbm::<Perlin>::new(sub_seed(seed, 3))
                .set_octaves(2)
                .set_frequency(1.0 / 650.0),
            ridge: RidgedMulti::<Perlin>::new(sub_seed(seed, 4))
                .set_octaves(3)
                .set_frequency(1.0 / 200.0),
            hills: Fbm::<Perlin>::new(sub_seed(seed, 5))
                .set_octaves(3)
                .set_frequency(1.0 / 90.0),
        }
    }

    /// Returns the height added above the coast profile at the warped position, `s` being the inland measure.
    pub fn above(&self, wx: f64, wz: f64, s: f32) -> f32 {
        let m = self.mount.get([wx, wz]) as f32;
        let h = smooth(
            0.2,
            0.8,
            (self.hills.get([wx, wz]) as f32 * 0.5 + 0.5).clamp(0.0, 1.0),
        );
        let rolling = h * (4.0 + 20.0 * smooth(-0.1, 0.25, m));
        let plateau = terrace(h * 25.0, 8.0);
        let low = rolling + (plateau - rolling) * smooth(-0.2, -0.38, m);
        let lift = smooth(0.02, 0.4, s);
        let mt = smooth(0.12, 0.38, m) * smooth(0.16, 0.5, s);
        if mt <= 0.0 {
            return low * lift;
        }
        let r = ((self.ridge.get([wx, wz]) as f32 + 1.0) * 0.5).clamp(0.0, 1.0);
        low * lift + mt * (24.0 + 90.0 * r * r)
    }
}
