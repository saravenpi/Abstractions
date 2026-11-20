//! Tests of the baked map of the light of the galaxies: its band lies in the plane of the disc
//! and is brightest toward the core, it bakes in time, and it is baked again when it should be.

use bevy::math::DVec3;

use super::march::{BYTES, FACES, SIDE, Source, bake, bake_at, rows};
use super::pack::{decode, encode};
use super::rig::{halfway, home};
use super::{Built, INSIDE, OUTSIDE, plan};
use crate::universe::{GLY, Glow};
use crate::world::cpu::busy;

/// A texel survives the bytes it is kept in to within a hundredth of a unit of light, from the
/// dark of the void to the glare of a core.
#[test]
fn the_light_survives_its_bytes() {
    for value in [0.0, 0.01, 0.2, 0.7, 1.5, 3.0] {
        let [r, ..] = encode(bevy::math::Vec3::splat(value));
        assert!((decode(r) - value).abs() < 0.02 + 0.03 * value, "{value}");
    }
}

/// Seen from half way out along a spoke, the band lies in the plane of the disc, within two
/// degrees.
#[test]
fn the_band_peaks_on_the_disc() {
    let all = halfway();
    let (mut sums, mut weights) = ([0.0f32; 60], [0.0f32; 60]);
    for (dir, weight, light) in &all {
        let bin = (dir.y.asin().to_degrees() + 30.0).floor();
        if (0.0..60.0).contains(&bin) {
            sums[bin as usize] += weight * light;
            weights[bin as usize] += weight;
        }
    }
    let means: Vec<f32> = sums
        .iter()
        .zip(&weights)
        .map(|(s, w)| s / w.max(1e-9))
        .collect();
    let peak = (0..60)
        .max_by(|a, b| means[*a].total_cmp(&means[*b]))
        .unwrap_or(0);
    assert!(
        (peak as f32 - 29.5).abs() <= 2.0,
        "the band peaks at {} degrees",
        peak as f32 - 29.5
    );
}

/// Seen from half way out along a spoke, the sky is brightest toward the real core, far above
/// what the other way holds.
#[test]
fn the_sky_is_brightest_toward_the_core() {
    let all = halfway();
    let cone = |toward: DVec3| {
        let seen: Vec<_> = all
            .iter()
            .filter(|(dir, ..)| dir.angle_between(toward) < 0.35)
            .collect();
        let share = seen.iter().map(|(_, w, l)| w * l).sum::<f32>();
        share / seen.iter().map(|(_, w, _)| w).sum::<f32>()
    };
    let (core, away) = (cone(DVec3::NEG_X), cone(DVec3::X));
    assert!(core > 3.0 * away, "core {core}, away {away}");
    let brightest = all.iter().max_by(|a, b| a.2.total_cmp(&b.2)).map(|t| t.0);
    let angle = brightest.map(|dir| dir.angle_between(DVec3::NEG_X));
    assert!(angle.is_some_and(|angle| angle < 0.35), "{angle:?}");
}

/// The bake of the home galaxy and every galaxy in sight takes under 300 ms on each of the six
/// threads it runs on at most, off the main thread, and fills the six faces. The time is the
/// processor time of the thread that does the share of one worker, so a machine busy with other
/// tests does not change it.
#[test]
fn the_bake_takes_under_300_ms() {
    let (universe, spec) = home();
    let (sources, _) = plan(&universe, spec.id, DVec3::new(0.4, 0.02, 0.1) * spec.radius);
    let start = busy();
    let share = rows(&sources, SIDE, (0, FACES));
    let took = busy() - start;
    assert_eq!(share.len(), SIDE);
    assert!(took < 0.3, "one worker of six took {took} s");
    assert_eq!(bake(&sources).len(), BYTES);
}

/// A map is stale after the camera moves 50 game light-years inside a galaxy, and after it
/// moves 2 percent of the distance to the nearest galaxy centre outside every galaxy, or into
/// another galaxy, and not before.
#[test]
fn the_map_is_baked_again_when_the_camera_has_moved() {
    let (universe, spec) = home();
    let inside = DVec3::new(0.3, 0.0, 0.1) * spec.radius;
    let (_, limit) = plan(&universe, spec.id, inside);
    assert_eq!(limit, INSIDE);
    let built = Built {
        galaxy: spec.id,
        at: inside,
        limit,
    };
    let moved = |gly: f64| inside + DVec3::Z * gly * GLY;
    assert!(!built.stale(spec.id, moved(49.0)) && built.stale(spec.id, moved(51.0)));
    let outside = DVec3::new(0.0, 0.5, 1.0).normalize() * 4.0 * spec.radius;
    let (sources, limit) = plan(&universe, spec.id, outside);
    let nearest = sources
        .iter()
        .map(|s| s.offset.length())
        .fold(f64::MAX, f64::min);
    assert!((limit - OUTSIDE * nearest).abs() < 1.0);
    let built = Built {
        galaxy: spec.id,
        at: outside,
        limit,
    };
    let along = |share: f64| outside + DVec3::X * share * nearest;
    assert!(!built.stale(spec.id, along(0.019)) && built.stale(spec.id, along(0.021)));
    let other = crate::universe::GalaxyId(spec.id.0 + bevy::math::IVec3::X);
    assert!(built.stale(other, outside));
}

/// The map of a lone galaxy far from the camera is dark where there is none, and the same
/// bake twice gives the same bytes.
#[test]
fn an_empty_sky_is_dark_and_a_bake_is_repeatable() {
    let (_, spec) = home();
    let far = Source {
        glow: Glow::of(&spec),
        offset: DVec3::X * 1.0e9 * GLY,
        inside: false,
    };
    let dark = bake_at(&[far], 16);
    assert!(
        dark.chunks(4)
            .all(|texel| texel[..3] == [0, 0, 0] && texel[3] == 255)
    );
    let near = Source {
        glow: Glow::of(&spec),
        offset: DVec3::X * 3.0 * spec.radius,
        inside: false,
    };
    assert_eq!(bake_at(&[near], 16), bake_at(&[near], 16));
}
