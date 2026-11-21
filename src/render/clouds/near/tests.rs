//! Tests of the cloud around the camera: its puffs are the drawn field given the depth of the
//! band, the very volume the whiteout reads, they stay at their places as the camera flies
//! through, a puff in daylight is never darker than a set share of the sunlit cloud, and the
//! lamps of the ship tint the puffs near them.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::fill::{FLOOR, Layer, PUFFS, REACH, lattice, tint, volume};
use super::lamps::{Lamp, glow};
use crate::render::scatter::luminance;
use crate::weather::{density_at, reached};

/// A temperate cloud band over an Earth, under a broken sky.
const LAYER: Layer = Layer {
    band: (1100.0, 3000.0),
    ground: 6.371e6,
    cells: 3540.0,
    seconds: 0.0,
    cover: 0.6,
};

/// Returns a planet-fixed direction under a towering part of the field, whose column reaches
/// the middle slice of the band.
fn cloudy() -> DVec3 {
    (0..4000)
        .map(|k| DVec3::new((k as f64 * 0.7).sin(), 0.3, (k as f64 * 0.7).cos()).normalize())
        .find(|dir| density_at(*dir, LAYER.cells, LAYER.seconds, LAYER.cover) > 0.8)
        .expect("a towering puff somewhere")
}

/// The volume is the drawn slices over the field: cloud from the base of the band up to the
/// highest slice the column reaches, nothing under the base, over that slice, over the top or in
/// a clear gap, so no puff floats where no slice is drawn.
#[test]
fn the_volume_is_the_drawn_cloud_at_its_height() {
    let dir = cloudy();
    let body = density_at(dir, LAYER.cells, LAYER.seconds, LAYER.cover);
    let (base, top) = LAYER.band;
    let highest = reached(body).expect("the base slice draws this puff");
    for share in [0.01, 0.5 * highest, highest - 0.01] {
        let altitude = base + share * (top - base);
        assert_eq!(volume(dir, altitude, &LAYER), 1.0, "{share} of the band");
    }
    let over = base + (highest + 0.05) * (top - base);
    assert_eq!(volume(dir, over, &LAYER), 0.0);
    assert_eq!(volume(dir, base - 50.0, &LAYER), 0.0);
    assert_eq!(volume(dir, top + 100.0, &LAYER), 0.0);
    let clear = Layer {
        cover: 0.0,
        ..LAYER
    };
    let gaps = (0..400).map(|k| DVec3::new(k as f64, 1.0, 0.5).normalize());
    assert!(
        gaps.into_iter()
            .any(|dir| volume(dir, 2050.0, &clear) == 0.0)
    );
}

/// The puffs within reach sit at points fixed to the planet: a camera that moves 10 m finds the
/// puffs it still reaches at the same places, so they stream past at its own speed. None is
/// farther than the reach and there are never more than the pool holds.
#[test]
fn the_puffs_stay_at_their_places() {
    let eye = cloudy() * (LAYER.ground + 2050.0);
    let (mut before, mut after) = (Vec::new(), Vec::new());
    lattice(eye, &LAYER, &mut before);
    lattice(eye + DVec3::new(10.0, 0.0, 0.0), &LAYER, &mut after);
    assert!(!before.is_empty() && before.len() <= PUFFS);
    assert!(before.iter().all(|puff| puff.at.distance(eye) <= REACH));
    let kept = after
        .iter()
        .filter(|puff| before.iter().any(|old| old.at == puff.at));
    assert!(kept.count() * 2 > after.len());
}

/// In daylight a puff at the base of the band is still at least [`FLOOR`] as bright as the
/// sunlit cloud, and at the top it is the cloud itself: the inside of a cloud is never a dark
/// veil.
#[test]
fn a_puff_is_never_dark_in_daylight() {
    let cloud = Vec3::new(0.85, 0.87, 0.9);
    for height in [0.0, 0.3, 1.0] {
        assert!(luminance(tint(cloud, height)) >= FLOOR * luminance(cloud) - 1.0e-6);
    }
    assert_eq!(tint(cloud, 1.0), cloud);
}

/// A red nav light of 600 lumens tints a puff 3 m away red and one 60 m away barely, by the
/// square of the distance, and lights nothing past its range; a headlight lights a puff in its
/// beam and none beside it.
#[test]
fn the_lamps_tint_the_puffs_near_them() {
    let nav = Lamp {
        at: Vec3::ZERO,
        power: Vec3::new(1.0, 0.05, 0.05) * 600.0,
        range: 40.0,
        cone: None,
    };
    let gain = 2.0;
    let near = glow(&[nav], Vec3::new(3.0, 0.0, 0.0), gain);
    let far = glow(&[nav], Vec3::new(30.0, 0.0, 0.0), gain);
    assert!(near.x > near.y * 10.0 && near.x > 0.002, "{near}");
    assert!((near.x / far.x - 100.0).abs() < 1.0, "{near} {far}");
    assert_eq!(glow(&[nav], Vec3::new(41.0, 0.0, 0.0), gain), Vec3::ZERO);
    let beam = Lamp {
        power: Vec3::splat(40_000.0),
        range: 200.0,
        cone: Some((Vec3::NEG_Z, 0.45f32.cos(), 0.3f32.cos())),
        ..nav
    };
    assert!(glow(&[beam], Vec3::new(0.0, 0.0, -30.0), gain).x > 0.0);
    assert_eq!(glow(&[beam], Vec3::new(30.0, 0.0, 0.0), gain), Vec3::ZERO);
}
