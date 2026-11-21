//! Tests of the cloud around the camera: its puffs are the drawn field given the depth of the
//! band, the very volume the whiteout reads, they stay at their places as the camera flies
//! through, and a puff in daylight is never darker than a set share of the sunlit cloud.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::super::cross::crossing_depth;
use super::fill::{FLOOR, Layer, PUFFS, REACH, lattice, tint, volume};
use crate::render::scatter::luminance;
use crate::weather::density_at;

/// A temperate cloud band over an Earth, under a broken sky.
const LAYER: Layer = Layer {
    band: (1100.0, 3000.0),
    ground: 6.371e6,
    cells: 3540.0,
    seconds: 0.0,
    cover: 0.6,
};

/// Returns a planet-fixed direction under a thick part of the field.
fn cloudy() -> DVec3 {
    (0..4000)
        .map(|k| DVec3::new((k as f64 * 0.7).sin(), 0.3, (k as f64 * 0.7).cos()).normalize())
        .find(|dir| density_at(*dir, LAYER.cells, LAYER.seconds, LAYER.cover) > 0.4)
        .expect("a thick puff somewhere")
}

/// The volume is the drawn field times the depth in the band: nothing under the base, over the
/// top or in a clear gap, and it rises as a camera climbs from the base into the puff.
#[test]
fn the_volume_is_the_drawn_cloud_at_its_height() {
    let dir = cloudy();
    let field = density_at(dir, LAYER.cells, LAYER.seconds, LAYER.cover);
    for altitude in [1500.0, 2050.0, 2600.0] {
        let want = crossing_depth(altitude, 1100.0, 3000.0) * field;
        assert!((volume(dir, altitude, &LAYER) - want).abs() < 1.0e-6);
    }
    assert_eq!(volume(dir, 900.0, &LAYER), 0.0);
    assert_eq!(volume(dir, 3100.0, &LAYER), 0.0);
    let clear = Layer {
        cover: 0.0,
        ..LAYER
    };
    let gaps = (0..400).map(|k| DVec3::new(k as f64, 1.0, 0.5).normalize());
    assert!(
        gaps.into_iter()
            .any(|dir| volume(dir, 2050.0, &clear) == 0.0)
    );
    let rising = [1150.0, 1400.0, 1700.0, 2050.0].map(|altitude| volume(dir, altitude, &LAYER));
    assert!(
        rising.windows(2).all(|pair| pair[1] > pair[0]),
        "{rising:?}"
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
