//! Tests of the brightness and the dust of the galaxy light at the real scale: the band of the
//! home galaxy is as bright as it was before the scale changed, and the dust of a galaxy dims its
//! core without blacking out the sky.

use bevy::math::DVec3;

use super::march::{SIDE, bake};
use super::plan;
use super::rig::{home, texels};
use super::tune::{REFERENCE, REFERENCE_SLACK};
use crate::universe::{Glow, LY};

/// Seen from 0.45 radii out along a spoke of the home galaxy, the brightest texel of the map
/// holds the light the bake of 0.10.1 gave from there, within 15 percent.
#[test]
fn the_home_band_is_as_bright_as_it_was() {
    let (universe, spec) = home();
    let (sources, _) = plan(&universe, spec.id, DVec3::X * 0.45 * spec.radius);
    let peak = texels(&bake(&sources), SIDE)
        .iter()
        .map(|texel| texel.2)
        .fold(0.0, f32::max);
    assert!(
        (peak / REFERENCE - 1.0).abs() < REFERENCE_SLACK,
        "the band peaks at {peak}, wanted {REFERENCE}"
    );
}

/// Returns the optical depth of the dust of `glow` along the straight line from `from` to `to`,
/// metres from the centre of its galaxy, summed in 4,000 steps.
fn depth(glow: &Glow, from: DVec3, to: DVec3) -> f64 {
    let steps = 4_000;
    let step = (to - from) / steps as f64;
    let sum: f64 = (0..steps)
        .map(|k| glow.dust(from + step * (k as f64 + 0.5)))
        .sum();
    sum * step.length() / LY
}

/// Through the whole disc along its plane, rim to rim past the core, the dust of a galaxy
/// 50,000 light-years in radius has an optical depth of 1 to 6, the mean of twelve directions,
/// as the Milky Way is opaque toward its core in its plane. Face on, through and beside the
/// centre, it is under 0.5. The depth grows with the radius, so a galaxy of another radius is
/// scaled to 50,000 light-years.
#[test]
fn the_dust_dims_the_plane_but_not_the_view_from_above() {
    let (_, spec) = home();
    let glow = Glow::of(&spec);
    let (rim, scale) = (1.5 * spec.radius, 50_000.0 * LY / spec.radius);
    let turns = (0..12).map(|k| f64::from(k) * std::f64::consts::TAU / 12.0);
    let along = |turn: f64| {
        let dir = DVec3::new(turn.cos(), 0.0, turn.sin()) * rim;
        depth(&glow, -dir, dir)
    };
    let edge_on = turns.map(along).sum::<f64>() / 12.0 * scale;
    assert!((1.0..6.0).contains(&edge_on), "edge on {edge_on}");
    for off in [0.0, 0.1] {
        let from = DVec3::new(off * spec.radius, -rim, 0.0);
        let face_on = depth(&glow, from, from * DVec3::new(1.0, -1.0, 1.0)) * scale;
        assert!(face_on < 0.5, "face on at {off} radii: {face_on}");
    }
}
