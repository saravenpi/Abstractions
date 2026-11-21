//! Tests of the band of the galaxy under the glow of the star: both are light and add, the band
//! washes out smoothly as the glow grows, with no notch, no hole and no new edge, its lanes and
//! clumps fade evenly, and the sky shader draws it by this rule.

use super::glare::{BAND_HOLD, glow, washed};

/// Returns what a pixel of the sky shows where the band has the luminance `light` under the
/// glow `around`: the glow plus the share of the band it leaves.
fn total(light: f32, around: f32) -> f32 {
    around + light * washed(light, around)
}

/// Luminances of the band from a faint lane to the bright core of a galaxy.
const BANDS: [f32; 7] = [0.0005, 0.003, 0.01, 0.03, 0.08, 0.2, 0.5];

/// Along a ray from the star out, over a band of any brightness and under a glow of any
/// strength, the sky never grows darker toward the star and never falls under the band or the
/// glow alone.
#[test]
fn the_band_never_dips_toward_the_star() {
    for power in [0.05_f32, 0.5, 1.0, 2.0] {
        for light in BANDS {
            let ray: Vec<f32> = (1..=600)
                .map(|step| total(light, glow((step as f32 * 0.2).to_radians().cos(), power)))
                .collect();
            let rising = ray
                .windows(2)
                .find(|pair| pair[1] > pair[0] * (1.0 + 1.0e-5));
            assert!(rising.is_none(), "{power} {light}: {rising:?}");
            assert!(
                ray.iter().all(|seen| *seen >= light * (1.0 - 1.0e-5)),
                "{power} {light}"
            );
        }
    }
}

/// Across the edge of the band, or from a lane to a clump, the sky brightens with the band and
/// by no more than the band does on black space: the glow adds no step of its own, and the
/// edge washes out as the glow grows.
#[test]
fn crossing_the_band_has_no_new_edge() {
    for around in [0.0_f32, 0.001, 0.01, 0.1, 1.0, 10.0] {
        let mut last = (0.0_f32, total(0.0, around));
        for step in 1..=500 {
            let light = step as f32 * 0.001;
            let seen = total(light, around);
            assert!(seen >= last.1, "{around}: {light}");
            assert!(
                seen - last.1 <= (light - last.0) * (1.0 + 1.0e-4),
                "{around}: {light}"
            );
            last = (light, seen);
        }
        let edge = total(0.02, around) - total(0.0, around);
        let washed_out = total(0.02, around * 2.0 + 0.01) - total(0.0, around * 2.0 + 0.01);
        assert!(washed_out < edge, "{around}");
    }
}

/// Under the glow every part of the band no brighter than [`BAND_HOLD`] keeps the same share of
/// its light, so a dust lane and a clump fade together; the share falls smoothly from whole to
/// half where the glow is as bright as the hold.
#[test]
fn the_lanes_and_clumps_fade_evenly() {
    for around in [0.0_f32, 0.003, 0.03, 0.3, 3.0] {
        let lane = washed(0.002, around);
        let clump = washed(BAND_HOLD, around);
        assert!((lane - clump).abs() < 1.0e-6, "{around}");
    }
    assert_eq!(washed(0.01, 0.0), 1.0);
    assert!((washed(0.01, BAND_HOLD) - 0.5).abs() < 1.0e-6);
    assert!(washed(0.01, 1.0) > 0.0);
}

/// The sky shader washes the band out by the rule tested here and adds the glow on top in the
/// colour of the star alone, so the band keeps its own colour.
#[test]
fn the_sky_shader_washes_the_band_by_the_rule() {
    let sky = include_str!("sky.wesl");
    for line in [
        format!("const BAND_HOLD: f32 = {BAND_HOLD:?};"),
        "let hold = max(light, BAND_HOLD);".to_string(),
        "return hold / (hold + max(glow, 0.0));".to_string(),
        "col = col + band * washed(dot(band, LUMA), glare);".to_string(),
        "col = col + sky.glow.rgb * (glare / max(dot(sky.glow.rgb, LUMA), 0.0001));".to_string(),
    ] {
        assert!(sky.contains(line.as_str()), "sky.wesl lacks `{line}`");
    }
}
