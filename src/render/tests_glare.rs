//! Tests of the glare of the star under no air: it is light that fades into black space with no
//! edge, nothing grows darker along a ray from the star out past a star it outshines, it is
//! gone under a whole sky and in the shadow of the planet, and the shaders draw it by this rule.

use super::glare::{GLOW_CORE, GLOW_PEAK, glow, outshone, strength};
use super::scatter::LUMA;
use super::starfield::shine;

/// Returns the cosine of `degrees` from the star.
fn off(degrees: f32) -> f32 {
    degrees.to_radians().cos()
}

/// Returns what a pixel shows along a ray from the star out, from a fifth of a degree to 120
/// degrees, for the glow of strength `power` under something of the luminance `light`: the glow
/// plus the share of that light it leaves.
fn along(power: f32, light: f32) -> Vec<f32> {
    (1..=600)
        .map(|step| {
            let around = glow(off(step as f32 * 0.2), power);
            around + light * outshone(light, around)
        })
        .collect()
}

/// The glow is bright light at the disc, falls all the way to black space and never rises
/// outward, so it has no edge and no ring.
#[test]
fn the_glow_fades_into_black_space() {
    assert!(glow(1.0, 1.0) >= GLOW_PEAK * 0.999);
    assert!(glow(off(1.0), 1.0) > 1.0);
    assert!(glow(off(90.0), 1.0) < 1.0e-4);
    let ray = along(1.0, 0.0);
    assert!(ray.windows(2).all(|pair| pair[1] <= pair[0]));
    assert!(ray.windows(2).all(|pair| pair[1] > 0.5 * pair[0]));
    assert!(GLOW_CORE < 2.0_f32.to_radians());
}

/// Along a ray from the star out, the glow and the light of a star it leaves never dip under
/// what lies farther out: no darker ring between the glow and the field, for every star from the
/// faintest to the brightest, at any strength. The band has its own rule, see `tests_glare_band`.
#[test]
fn nothing_grows_darker_toward_the_star() {
    let stars = [-4.6_f32, -3.0, -1.0, 0.0, 1.0, 2.0].map(|flux| shine(flux).1);
    for power in [0.05_f32, 0.5, 1.0, 2.0] {
        for light in &stars {
            let ray = along(power, *light);
            let rising = ray.windows(2).find(|pair| pair[1] > pair[0] + 1.0e-5);
            assert!(rising.is_none(), "{power} {light}: {rising:?}");
            assert!(
                ray.iter().all(|seen| *seen >= *light - 1.0e-5),
                "{power} {light}"
            );
        }
    }
}

/// Close to the star its glow outshines even the brightest star of the field, far out it leaves
/// every star whole, and in the shadow of the planet there is no glow to outshine anything.
#[test]
fn the_glow_outshines_the_stars_near_the_star() {
    let brightest = shine(1.0).1;
    assert_eq!(outshone(brightest, glow(off(0.5), 1.0)), 0.0);
    assert!(outshone(shine(-4.6).1, glow(off(30.0), 1.0)) > 0.99);
    assert_eq!(outshone(brightest, glow(off(0.5), 0.0)), 1.0);
    assert_eq!(outshone(brightest, 0.0), 1.0);
}

/// The glow is whole in space, at the reference exposure and in full light, gone under a whole
/// sky, where the dome draws the glow of the air the owner tuned, and in the shadow of the planet,
/// and it follows the exposure of the camera.
#[test]
fn the_glow_needs_open_space_and_the_light_of_the_star() {
    assert_eq!(strength(1.0, 0.0, 1.0), 1.0);
    assert_eq!(strength(1.0, 1.0, 1.0), 0.0);
    assert_eq!(strength(0.0, 0.0, 1.0), 0.0);
    assert!(strength(1.0, 0.0, 0.7) < strength(1.0, 0.0, 1.0));
    assert!(strength(1.0, 0.3, 1.0) > strength(1.0, 0.6, 1.0));
}

/// Both shaders draw the glow by the rule tested here, and the star shader outshines by it.
#[test]
fn the_shaders_have_the_glow_of_the_rule() {
    let triple = format!("vec3<f32>({:?}, {:?}, {:?})", LUMA.x, LUMA.y, LUMA.z);
    let lines = [
        format!("const LUMA: vec3<f32> = {triple};"),
        format!("const GLOW_PEAK: f32 = {GLOW_PEAK:?};"),
        format!("const GLOW_CORE: f32 = {GLOW_CORE:?};"),
        "/ (GLOW_CORE * GLOW_CORE);".to_string(),
        "/ pow(1.0 + spread, 1.5);".to_string(),
    ];
    let sky = include_str!("sky.wesl");
    let stars = include_str!("starfield.wesl");
    for line in &lines {
        assert!(sky.contains(line.as_str()), "sky.wesl lacks `{line}`");
        assert!(
            stars.contains(line.as_str()),
            "starfield.wesl lacks `{line}`"
        );
    }
    assert!(stars.contains("return clamp(1.0 - max(glow, 0.0) / max(light, 0.000001), 0.0, 1.0);"));
    assert!(stars.contains("let kept = outshone(dot(light, LUMA), halo(seen.dir));"));
}
