//! Tests of what a grain of dust looks like and where there is dust at all: a faint grey line
//! that shows only while it moves, and none outside a galaxy.

use bevy::prelude::*;

use super::Lens;
use super::dust::{
    GRAIN, GRAIN_ASPECT, GRAIN_LIGHT, GRAIN_PACE, GRAIN_REACH, MOTES, line, mote, shown,
};

/// A grain that stands still on the view shows nothing, its light rises with the angle it
/// crosses during one exposure, and it never shows more than the cap, however near it passes.
#[test]
fn a_grain_shows_only_while_it_moves_and_stays_faint() {
    assert_eq!(shown(1.0, 0.0), 0.0);
    let spans: Vec<f32> = (0..=100).map(|i| i as f32 * GRAIN_PACE / 50.0).collect();
    let lights: Vec<f32> = spans.iter().map(|span| shown(1.0, *span)).collect();
    assert!(
        lights.windows(2).all(|pair| pair[1] >= pair[0]),
        "{lights:?}"
    );
    assert!(
        lights
            .windows(2)
            .all(|pair| pair[1] - pair[0] < 0.05 * GRAIN_LIGHT)
    );
    assert_eq!(shown(1.0, GRAIN_PACE), GRAIN_LIGHT);
    assert_eq!(shown(7.0, 9.0), GRAIN_LIGHT);
    assert_eq!(shown(0.5, 9.0), 0.5 * GRAIN_LIGHT);
    for index in 0..MOTES {
        let grain = mote(index, 12.3 + index as f32 * 0.21, 1.0, Vec3::NEG_Z);
        assert!((0.0..=1.0).contains(&grain.light), "{index}: {grain:?}");
    }
}

/// A grain is a line, never a dot: at any speed it is at least four times as long as it is
/// wide, it grows with the path it crosses, and it stays short.
#[test]
fn a_grain_is_a_short_line() {
    for step in 0..=200 {
        let (wide, long) = line(step as f32 * 0.001);
        assert_eq!(wide, GRAIN);
        assert!(long >= wide * GRAIN_ASPECT && long <= GRAIN_REACH, "{long}");
    }
    assert!(line(0.03).1 > line(0.02).1 && line(1.0).1 == GRAIN_REACH);
    const { assert!(GRAIN_REACH < 0.05, "a grain is a short line") };
}

/// The dust is the medium of a galaxy: all of it inside the star disc, less as the camera
/// leaves the disc, grain by grain, and none between the galaxies, whatever the drive does.
#[test]
fn there_is_no_dust_between_the_galaxies() {
    let lens = Lens {
        dust: 1.0,
        ..Lens::default()
    };
    assert_eq!((lens.medium(1.0), lens.medium(0.0)), (1.0, 0.0));
    assert_eq!(Lens::default().medium(1.0), 0.0);
    let lit = |band: f32| {
        let density = lens.medium(band);
        let grain = |index: &u32| mote(*index, 10.5, density, Vec3::NEG_Z).light > 0.0;
        (0..MOTES).filter(grain).count()
    };
    let counts = [0.0, 0.25, 0.5, 0.75, 1.0].map(lit);
    assert!(counts[0] == 0 && counts[4] as f32 > 0.9 * MOTES as f32);
    assert!(
        counts.windows(2).all(|pair| pair[1] >= pair[0]),
        "{counts:?}"
    );
    assert!(counts[1] < counts[3], "{counts:?}");
}

/// The shader draws the grains with the constants of the rules tested here, in grey, and the
/// field is a few grains, not a second sky.
#[test]
fn the_shader_draws_the_grains_by_these_rules() {
    let shader = include_str!("../starfield.wesl");
    let lines = [
        format!("const GRAIN: f32 = {GRAIN:?};"),
        format!("const GRAIN_ASPECT: f32 = {GRAIN_ASPECT:?};"),
        format!("const GRAIN_REACH: f32 = {GRAIN_REACH:?};"),
        format!("const GRAIN_LIGHT: f32 = {GRAIN_LIGHT:?};"),
        format!("const GRAIN_PACE: f32 = {GRAIN_PACE:?};"),
        "let half = clamp(0.5 * span, GRAIN * GRAIN_ASPECT, GRAIN_REACH);".to_string(),
        "let moving = smoothstep(0.0, GRAIN_PACE, span);".to_string(),
        "let grey = GRAIN_LIGHT * light * moving * stars.params.x * stars.params.z;".to_string(),
        "vec4<f32>(vec3<f32>(grey), vertex.color.a)".to_string(),
    ];
    for line in &lines {
        assert!(
            shader.contains(line.as_str()),
            "starfield.wesl lacks `{line}`"
        );
    }
    const { assert!(MOTES <= 200, "a few grains, not a star field") };
}

/// Nothing in the sky of a warp runs on a cycle: the shader reads no clock, and the sway of
/// the stars along their streaks is gone.
#[test]
fn the_sky_of_a_warp_has_no_cycle() {
    let shader = include_str!("../starfield.wesl");
    for gone in [
        "crawl",
        "CRAWL",
        "TWINKLE",
        "sway",
        "globals",
        "stars.flow.w",
    ] {
        assert!(!shader.contains(gone), "starfield.wesl still has `{gone}`");
    }
}
