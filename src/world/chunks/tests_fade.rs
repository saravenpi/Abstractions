//! Tests of the fade maths against the dither of Bevy's mesh shader, redone here on the CPU.

use bevy::camera::visibility::VisibilityRange;

use super::dither::{FADE_SECONDS, LEVELS, SPAN, Way, level, range, ranged, span};

/// The 4 by 4 Bayer thresholds of `visibility_range_dither` in Bevy's `pbr_functions`.
const BAYER: [i32; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

/// Returns the dither level Bevy's `get_visibility_range_dither_level` gives a mesh with `range`
/// seen from `distance` metres.
fn dither(range: &VisibilityRange, distance: f32) -> i32 {
    let far = distance >= range.end_margin.start;
    let (offset, bounds) = match far {
        true => (0, &range.end_margin),
        false => (-16, &range.start_margin),
    };
    let level = ((distance - bounds.start) / (bounds.end - bounds.start) * 16.0).round();
    offset + (level as i32).clamp(0, 16)
}

/// Returns true when Bevy's `visibility_range_dither` draws the pixel of Bayer `threshold` at
/// dither level `dither`.
fn drawn(dither: i32, threshold: i32) -> bool {
    let culled = dither <= -16 || dither >= 16;
    let fading_out = dither >= 0 && dither + threshold >= 16;
    let fading_in = dither < 0 && 1 + dither + threshold <= 0;
    dither == 0 || !(culled || fading_out || fading_in)
}

/// Returns how many of the sixteen Bayer pixels a mesh fading `way` shows at `level`, seen
/// from `distance` metres.
fn pixels(way: Way, level: u8, distance: f32) -> usize {
    let dither = dither(&range(way, level), distance);
    BAYER.iter().filter(|t| drawn(dither, **t)).count()
}

/// A fade shows one more sixteenth of the pixels per level, whatever the distance of the
/// camera, and the camera never culls it on the way.
#[test]
fn fade_level_sets_the_share_of_pixels() {
    for distance in [0.0, 3.0, 250.0, 9000.0, 120_000.0] {
        for level in 0..=LEVELS {
            let share = usize::from(level);
            assert_eq!(pixels(Way::In, level, distance), share);
            assert_eq!(pixels(Way::Hide, level, distance), 16 - share);
            assert_eq!(pixels(Way::Leave, level, distance), 16 - share);
            let seen = range(Way::In, level).is_visible_at_all(distance);
            assert!(seen, "level {level} at {distance} m");
        }
    }
}

/// What fades out and what fades in at the same level share the pixels between them: every
/// pixel is drawn exactly once, so a swap shows no hole and no doubled surface.
#[test]
fn fades_in_and_out_fill_every_pixel_once() {
    for level in 0..=LEVELS {
        let (coming, going) = (range(Way::In, level), range(Way::Hide, level));
        for threshold in BAYER {
            let new = drawn(dither(&coming, 700.0), threshold);
            let old = drawn(dither(&going, 1900.0), threshold);
            assert_ne!(new, old, "level {level} threshold {threshold}");
        }
    }
}

/// A fade runs its sixteen levels over its duration and stays done afterwards.
#[test]
fn fade_runs_for_its_duration() {
    assert_eq!(level(0.0), 0);
    assert_eq!(level(-1.0), 0);
    assert_eq!(level(FADE_SECONDS * 0.5), LEVELS / 2);
    assert_eq!(level(FADE_SECONDS), LEVELS);
    assert_eq!(level(FADE_SECONDS * 30.0), LEVELS);
    let steps = (0..=40).map(|i| level(FADE_SECONDS * i as f32 / 40.0));
    assert!(steps.collect::<Vec<_>>().windows(2).all(|w| w[0] <= w[1]));
}

/// On a planet of 6,371 km a node can lie 13,000 km from the camera, which the fixed margin of a
/// small planet cannot carry: its level would move with the distance. The margin that grows with
/// the planet and the altitude keeps every level exact at every distance up to the far side.
#[test]
fn the_margin_grows_with_the_planet_and_the_altitude() {
    let radius = 6.371e6;
    assert_eq!(span(2.0, 6.0e3), SPAN);
    assert!(span(2.0, radius) > span(2.0, 6.0e3));
    assert!(span(2.0e7, radius) > span(2.0, radius));
    let wide = span(0.0, radius);
    for (altitude, level) in [(0.0, 0), (30.0, 3), (1.0e5, 8), (3.0e6, 16)] {
        let wide = span(altitude, radius);
        for distance in [3.0, 9.0e3, 6.0e5, 6.4e6, 1.2e7, 1.9e7] {
            let drawn = |way| {
                let range = ranged(way, level, wide);
                let share = ((distance - range.start_margin.start) / wide * 16.0).round() as i32;
                share.clamp(0, 16)
            };
            assert_eq!(
                drawn(Way::In),
                i32::from(level),
                "{altitude} m, {distance} m"
            );
        }
    }
    assert!(wide >= 64.0 * 2.0 * radius as f32);
}
