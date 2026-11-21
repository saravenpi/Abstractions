//! The dither of the colour banding: it only works the seam between two levels, so a flat sky
//! stays clean, and a smooth bright gradient bands into steps no wider than a level.

/// Returns the value of the constant `name` declared in the shader.
fn constant(name: &str) -> f32 {
    let source = include_str!("pixelate.wgsl");
    let start = source.find(&format!("const {name}: f32 = ")).expect(name) + name.len() + 13;
    let end = start + source[start..].find(';').expect("end of constant");
    source[start..end].trim().parse().expect("a number")
}

/// Returns the dither threshold of the shader at texel `(x, y)`: interleaved gradient noise.
fn threshold(x: usize, y: usize) -> f32 {
    let line = x as f32 * constant("IGN_X") + y as f32 * constant("IGN_Y");
    (constant("IGN_SCALE") * line.fract()).fract()
}

/// Returns how far, in levels, the shader moves a channel of perceptual value `p` at texel
/// `(x, y)` from its true value: banded and dithered, then softened toward the true value on
/// bright colours.
fn error(p: f32, x: usize, y: usize) -> f32 {
    let (levels, white, soft) = (
        super::tune::LEVELS,
        constant("WHITE_DITHER"),
        constant("WHITE_SOFT"),
    );
    let strength = 1.0 + (white - 1.0) * p;
    let scaled = p * levels;
    let base = scaled.floor();
    let width = 2.0 * constant("DITHER_BAND") * strength;
    let share = ((scaled - base - 0.5) / width + 0.5).clamp(0.0, 1.0);
    let banded = base + f32::from(threshold(x, y) < share);
    (banded - scaled) * (1.0 - soft * p)
}

/// The smooth bright gradient of the sea seen from above, averaged over 4 by 4 pixels the way
/// the eye does, stays within half a level of the true value: no wide rings. The dither of the
/// whole level swung by a quarter of a level, the seam dither swings as far as plain rounding
/// does, in return for a sky without grain.
#[test]
fn bright_gradients_do_not_band_into_rings() {
    let shade = |x: usize, y: usize| error(0.55 + 0.12 * (x as f32 / 480.0).powi(2), x, y);
    let swing = (0..476).map(|x0| {
        let cells = (0..16).map(|k| shade(x0 + k % 4, k / 4));
        cells.sum::<f32>() / 16.0
    });
    let (low, high) = swing.fold((f32::MAX, f32::MIN), |(a, b), e| (a.min(e), b.max(e)));
    assert!(
        high - low < 0.55,
        "the gradient swings by {} levels",
        high - low
    );
}

/// On flat bright sky colours the grain the dither leaves, the spread of the pixels around
/// their true value averaged over every shade between two levels, stays well under the 0.36
/// level the plain 2 by 2 checker of the old full-strength Bayer dither showed.
#[test]
fn flat_bright_colours_keep_a_faint_grain() {
    let spread = |f: f32| {
        let p = (17.0 + f) / super::tune::LEVELS;
        let all: Vec<f32> = (0..1024).map(|k| error(p, k % 32, k / 32)).collect();
        let mean = all.iter().sum::<f32>() / 1024.0;
        (all.iter().map(|e| (e - mean).powi(2)).sum::<f32>() / 1024.0).sqrt()
    };
    let grain = (0..40).map(|k| spread(k as f32 / 40.0)).sum::<f32>() / 40.0;
    assert!(grain < 0.2, "the grain spreads by {grain} levels");
}

/// The thresholds fill the unit range evenly: over 64 by 64 texels every sixteenth of it holds
/// between 5 and 7.5 percent of them, so no shade is dithered more than another.
#[test]
fn thresholds_cover_every_level_evenly() {
    let mut buckets = [0u32; 16];
    for k in 0..4096 {
        let t = threshold(k % 64, k / 64);
        assert!((0.0..1.0).contains(&t), "{t}");
        buckets[(t * 16.0) as usize] += 1;
    }
    for count in buckets {
        assert!((205..=307).contains(&count), "{buckets:?}");
    }
}
