//! Tests of the daylight rule: it never falls as the star climbs, it is dark under the horizon,
//! it is the light the palette always had, and the shader holds the same constants.

use bevy::prelude::*;

use super::{Daylight, NIGHT_EDGE, SKY_EDGE, SUN_EDGE, daylight};
use crate::render::atmosphere::SHADOW_EDGE;
use crate::render::celestial::smoothstep;
use crate::render::palette::{NIGHT_AMBIENT_COLOR, Palette};
use crate::render::scatter::{DEPTH, HORIZON_MASS, LUMA, luminance, peak, transmit};

/// The shader that holds the rule for the GPU.
const SHADER: &str = include_str!("../daylight.wesl");

/// Star colours the rule is checked under: white, red and blue.
const STARS: [Vec3; 3] = [
    Vec3::ONE,
    Vec3::new(1.0, 0.5, 0.3),
    Vec3::new(0.7, 0.8, 1.0),
];

/// Returns the numbers of the constant `name` of the shader, in the order they are written.
fn shader_constant(name: &str) -> Vec<f32> {
    let start = format!("const {name}:");
    let line = SHADER.lines().find(|line| line.starts_with(&start));
    let value = line.and_then(|line| line.split('=').nth(1)).unwrap_or("");
    let bare = value.replace("vec2<f32>", "").replace("vec3<f32>", "");
    let numbers = bare.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'));
    numbers.filter_map(|number| number.parse().ok()).collect()
}

/// Returns the brightness of the direct light of `light` against the open star.
fn share(light: &Daylight) -> f32 {
    light.through * light.risen
}

/// As the star climbs from straight under to straight over, the brightness of the direct light
/// and the day sky only ever grow, under every star and through thin, home and heavy air, and
/// the tint stays a colour whose strongest channel is 1.
#[test]
fn daylight_never_falls_as_the_star_climbs() {
    for star in STARS {
        for depth in [0.2, 1.0, 2.5] {
            let steps: Vec<Daylight> = (0..=400)
                .map(|step| daylight(step as f32 / 200.0 - 1.0, star, depth))
                .collect();
            for pair in steps.windows(2) {
                assert!(share(&pair[1]) >= share(&pair[0]), "{star} {depth}");
                assert!(pair[1].sky >= pair[0].sky, "{star} {depth}");
                assert_eq!(pair[1].tint.max_element(), 1.0, "{star} {depth}");
                assert!(pair[1].sun().max_element() <= share(&pair[1]) + 1.0e-6);
            }
        }
    }
}

/// Once the star is under the horizon by more than the lower edge of its disc there is no direct
/// light at all, a little above it there is, and the sky keeps a glow longer than the disc.
#[test]
fn no_direct_light_under_the_horizon() {
    for star in STARS {
        for height in [-1.0, -0.5, -0.1, SUN_EDGE.0] {
            let dark = daylight(height, star, 1.0);
            assert_eq!((dark.risen, dark.sun()), (0.0, Vec3::ZERO), "{height}");
        }
        assert!(share(&daylight(SUN_EDGE.0 + 0.01, star, 1.0)) > 0.0);
        assert_eq!(daylight(SUN_EDGE.1, star, 1.0).risen, 1.0);
        assert!(daylight(SUN_EDGE.0, star, 1.0).sky > 0.0);
        assert_eq!(daylight(SKY_EDGE.0, star, 1.0).sky, 0.0);
    }
}

/// The rule gives, bit for bit, the direct light and the day share the palette computed by
/// itself before the rule existed, and the palette now carries exactly what the rule gives.
#[test]
fn daylight_is_the_light_the_palette_had() {
    for star in STARS {
        for step in 0..=400 {
            let height = step as f32 / 200.0 - 1.0;
            let through = star * transmit(height, 1.0);
            let full = luminance(star * transmit(1.0, 1.0)).max(1.0e-6);
            let lux =
                12_000.0 * (luminance(through) / full).min(1.0) * smoothstep(-0.06, 0.03, height);
            let light = daylight(height, star, 1.0);
            assert_eq!(light.tint, peak(through), "{height}");
            assert_eq!(Palette::SPACE_LUX * light.through * light.risen, lux);
            assert_eq!(light.sky, smoothstep(-0.12, 0.25, height), "{height}");
            let palette = Palette::lit(height, star);
            assert_eq!((palette.sun_color, palette.sun_lux), (light.tint, lux));
            assert_eq!(
                palette.ambient_brightness,
                40.0 + (350.0 - 40.0) * light.sky
            );
            assert_eq!(palette.moon_lux, 350.0 * smoothstep(-0.02, 0.3, -height));
        }
    }
}

/// Every constant the shader shares with the Rust side holds the same numbers under the same name.
#[test]
fn the_shader_holds_the_same_constants() {
    let pair = |edge: (f32, f32)| vec![edge.0, edge.1];
    assert_eq!(shader_constant("DEPTH"), DEPTH.to_array());
    assert_eq!(shader_constant("HORIZON_MASS"), [HORIZON_MASS]);
    assert_eq!(shader_constant("LUMA"), LUMA.to_array());
    assert_eq!(shader_constant("SUN_EDGE"), pair(SUN_EDGE));
    assert_eq!(shader_constant("SKY_EDGE"), pair(SKY_EDGE));
    assert_eq!(shader_constant("NIGHT_EDGE"), pair(NIGHT_EDGE));
    assert_eq!(shader_constant("SHADOW_EDGE"), pair(SHADOW_EDGE));
    assert_eq!(
        shader_constant("NIGHT_AMBIENT_COLOR"),
        NIGHT_AMBIENT_COLOR.to_array()
    );
    assert_eq!(shader_constant("NIGHT_AMBIENT"), [Palette::NIGHT_AMBIENT]);
    assert_eq!(shader_constant("DAY_AMBIENT"), [Palette::DAY_AMBIENT]);
    assert_eq!(shader_constant("SPACE_AMBIENT"), [Palette::SPACE_AMBIENT]);
    assert_eq!(shader_constant("SPACE_LUX"), [Palette::SPACE_LUX]);
}
