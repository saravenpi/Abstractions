//! The refill budget: when a new patch is painted, how many texels a walk costs, how long
//! one patch takes, how little a moved patch samples, and where a fast player's patch goes.

use std::time::Instant;

use bevy::math::{DVec3, IVec2, Vec2};

use crate::menus::hud::minimap::ahead::{ahead, cover, next};
use crate::menus::hud::minimap::paint::{BYTES, paint, repaint};
use crate::menus::hud::minimap::patch::{DRIFT, Patch, TEXELS};
use crate::menus::hud::minimap::range::RANGES;
use crate::world::Globe;

/// Radius in metres of the planet the tests walk on.
const RADIUS: f64 = 6000.0;

/// Returns the unit direction `metres` north of the point of the equator the tests start at.
fn north_of_start(metres: f64) -> DVec3 {
    let angle = metres / RADIUS;
    DVec3::new(angle.cos(), angle.sin(), 0.0)
}

/// A patch reaches two ranges from its middle, has north up and east to the right, and finds
/// a point again where it put it.
#[test]
fn a_patch_is_a_metric_square_with_north_up() {
    let patch = Patch::around(DVec3::X, RADIUS, 250.0);
    assert_eq!(patch.half(), 500.0);
    let north = patch.locate(north_of_start(100.0));
    assert!(north.distance(Vec2::new(0.0, 100.0)) < 1e-2, "{north}");
    let east = patch.locate(DVec3::new(1.0, 0.0, -0.01).normalize());
    assert!(east.x > 59.0 && east.y.abs() < 1e-3, "{east}");
    let texel = 1000.0 / TEXELS as f32;
    let corner = patch.locate(patch.dir(0, 0));
    assert!(corner.distance(Vec2::new(-500.0, 500.0) + Vec2::new(0.5, -0.5) * texel) < 1e-2);
    let last = patch.locate(patch.dir(TEXELS - 1, TEXELS - 1));
    assert!(last.distance(Vec2::new(500.0, -500.0) + Vec2::new(-0.5, 0.5) * texel) < 1e-2);
    let pole = Patch::around(DVec3::Y, RADIUS, 250.0);
    assert!(pole.locate(pole.dir(TEXELS / 2, 0)).y > 490.0);
}

/// Nothing is painted while the player stays near the middle of the patch or turns. Past the
/// drift a patch is painted around the player, moved by whole texels so the patterns hold
/// still, and a change of range paints a fresh one.
#[test]
fn a_patch_is_painted_only_when_due() {
    let first = next(None, DVec3::X, RADIUS, 250.0).unwrap();
    assert_eq!(first, Patch::around(DVec3::X, RADIUS, 250.0));
    let near = north_of_start(f64::from(DRIFT * 250.0) - 1.0);
    assert_eq!(next(Some(first), DVec3::X, RADIUS, 250.0), None);
    assert_eq!(next(Some(first), near, RADIUS, 250.0), None);
    let far = north_of_start(f64::from(DRIFT * 250.0) + 1.0);
    let moved = next(Some(first), far, RADIUS, 250.0).unwrap();
    let texel = 1000.0 / TEXELS as f32;
    assert!(moved.locate(far).length() < texel, "{}", moved.locate(far));
    let shift = moved.locate(first.chart.origin) / texel;
    assert!(
        shift.distance(shift.round()) < 1e-2 && shift.y.round() == -7.0,
        "{shift}"
    );
    assert!(moved.chart.east.distance(first.chart.east) < 1e-9);
    let wider = next(Some(first), DVec3::X, RADIUS, 800.0).unwrap();
    assert_eq!((wider.range, wider.half()), (800.0, 1600.0));
}

/// Returns how many patches a straight walk of `metres` north paints at `range`, a step of
/// two metres at a time, each patch being ready before the next step.
fn refills(metres: f64, range: f32) -> usize {
    let mut shown = None;
    let mut painted = 0;
    for step in 0..=(metres / 2.0) as usize {
        if let Some(patch) = next(shown, north_of_start(step as f64 * 2.0), RADIUS, range) {
            (shown, painted) = (Some(patch), painted + 1);
        }
    }
    painted
}

/// The budget of a walk: one patch at the start and one more every 0.3 range walked, so a
/// sprint at 8 m/s on the nearest range paints under 1,200 texels per second, 20 per frame,
/// and none of them on the main thread.
#[test]
fn a_walk_stays_within_the_refill_budget() {
    let walked = 1500.0;
    for range in [RANGES[0], RANGES[1]] {
        let most = 2 + (walked / f64::from(DRIFT * range)) as usize;
        let painted = refills(walked, range);
        assert!(
            painted >= 2 && painted <= most,
            "{painted} patches at {range} m"
        );
    }
    let per_second = refills(walked, RANGES[0]) as f64 * (TEXELS * TEXELS) as f64 / (walked / 8.0);
    assert!(per_second < 1200.0, "{per_second} texels per second");
}

/// One patch is 9,216 heights. Painting it is timed here and must stay far under the time a
/// player needs to cross the margin of the patch on screen, even in a test build.
#[test]
fn a_patch_is_painted_within_its_budget() {
    let globe = Globe::new(7, RADIUS, 22.0);
    let patch = Patch::around(DVec3::new(0.6, 0.3, -0.74).normalize(), RADIUS, 250.0);
    let started = Instant::now();
    let pixels = paint(&globe, &patch);
    let spent = started.elapsed();
    let texels = pixels.len() / 4;
    let each = spent.as_secs_f64() * 1e6 / texels as f64;
    println!("minimap patch: {texels} texels in {spent:?}, {each:.2} us per texel");
    assert_eq!(texels, TEXELS * TEXELS);
    assert!(spent.as_secs_f64() < 2.0, "{spent:?}");
}

/// Radius in metres of a planet of real size.
const EARTH: f64 = 6.371e6;

/// A patch moved 750 m north on a planet of real size takes seven rows of texels from the one
/// before and samples only its new edge: under a tenth of the samples of a whole patch, the new
/// rows exactly as a whole paint gives them and nearly every other texel too. On a planet a few
/// kilometres wide the ground curves too much to reuse texels, and the patch is painted whole.
#[test]
fn a_moved_patch_samples_only_its_new_edge() {
    let globe = Globe::new(7, EARTH, 9.8);
    let first = Patch::around(DVec3::new(0.6, 0.3, -0.74).normalize(), EARTH, 2500.0);
    let old = paint(&globe, &first);
    let moved = first.moved(Vec2::new(0.0, 750.0));
    assert_eq!(moved.texels_in(&first), Some(IVec2::new(0, -7)));
    let before = Globe::sampled();
    let whole = paint(&globe, &moved);
    let full = Globe::sampled() - before;
    let edge = repaint(&globe, &moved, Some((&first, &old)));
    let taken = Globe::sampled() - before - full;
    assert!(taken * 10 < full, "{taken} samples of {full}");
    let rows = 7 * TEXELS * BYTES;
    assert_eq!(edge[..rows], whole[..rows]);
    let same = edge
        .chunks_exact(BYTES)
        .zip(whole.chunks_exact(BYTES))
        .filter(|(a, b)| a == b)
        .count();
    assert!(same * 100 >= TEXELS * TEXELS * 99, "{same} texels match");
    let small = Patch::around(DVec3::X, RADIUS, 2000.0);
    assert_eq!(small.moved(Vec2::new(0.0, 600.0)).texels_in(&small), None);
}

/// A fast player's next patch is centred ahead of it, never past the drift of a patch, and a
/// patch covers less of the map the farther the player is from its middle.
#[test]
fn a_fast_player_is_painted_ahead() {
    let at = DVec3::X * EARTH;
    let north = DVec3::Y * 300.0;
    let patch = Patch::around(DVec3::X, EARTH, 2500.0);
    let led = patch.locate(ahead(at, north, 2500.0));
    assert!((led.y - 150.0).abs() < 0.5 && led.x.abs() < 0.5, "{led}");
    let capped = patch.locate(ahead(at, north * 100.0, 2500.0));
    assert!((capped.y - DRIFT * 2500.0).abs() < 0.5, "{capped}");
    let middle = cover(&patch, DVec3::X);
    assert!(
        (middle - 5000.0 / std::f32::consts::SQRT_2).abs() < 0.5,
        "{middle}"
    );
    let off = cover(&patch, ahead(at, north * 100.0, 2500.0));
    assert!(
        (middle - off - DRIFT * 2500.0 / std::f32::consts::SQRT_2).abs() < 0.5,
        "{off}"
    );
}
