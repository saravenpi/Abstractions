//! The dither of the colour banding: smooth bright gradients do not band into wide rings.

/// Returns the value of the constant `name` declared in the shader.
fn constant(name: &str) -> f32 {
    let source = include_str!("pixelate.wgsl");
    let start = source.find(&format!("const {name}: f32 = ")).expect(name) + name.len() + 13;
    let end = start + source[start..].find(';').expect("end of constant");
    source[start..end].trim().parse().expect("a number")
}

/// The smooth bright gradient of the sea seen from above, banded and dithered the way the
/// shader does it and averaged over 4 by 4 pixels the way the eye does, stays within a quarter
/// of a level of the true value: no wide rings. The old strength of 0.4 swung by half a level.
#[test]
fn bright_gradients_do_not_band_into_rings() {
    let (levels, white) = (24.0f32, constant("WHITE_DITHER"));
    let bayer = [
        0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0,
    ];
    let shade = |x: usize, y: usize| {
        let p = 0.55 + 0.12 * (x as f32 / 480.0).powi(2);
        let strength = 1.0 + (white - 1.0) * p;
        let d = (bayer[(y & 3) * 4 + (x & 3)] + 0.5) / 16.0;
        (p * levels + 0.5 + (d - 0.5) * strength).floor() - p * levels
    };
    let swing = (0..476).map(|x0| {
        let cells = (0..16).map(|k| shade(x0 + k % 4, k / 4));
        cells.sum::<f32>() / 16.0
    });
    let (low, high) = swing.fold((f32::MAX, f32::MIN), |(a, b), e| (a.min(e), b.max(e)));
    assert!(
        high - low < 0.25,
        "the gradient swings by {} levels",
        high - low
    );
}
