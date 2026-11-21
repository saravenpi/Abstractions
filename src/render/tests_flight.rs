//! Tests that flight through air draws nothing around the camera: the shell around the view
//! carries the light of a warp only, and inside a cloud the fog thickens with the cloud the field
//! holds at the camera, not with a pattern of its own.

use bevy::prelude::*;

use super::entry::FxUniform;
use super::fog::white;
use super::warp::Lens;

/// At rest and in any flight without a warp the shell around the camera draws nothing, and its
/// shader holds no lanes, wisps or streaks: only the light of a warp lights it.
#[test]
fn no_flight_effect_is_drawn_on_the_camera_shell() {
    let calm = Lens::default();
    assert!(!FxUniform::of(&calm, Vec3::NEG_Z).lit());
    let warp = Lens { core: 0.5, ..calm };
    assert!(FxUniform::of(&warp, Vec3::NEG_Z).lit());
    let shader = include_str!("entry.wesl");
    for pattern in ["lanes", "wisp", "streak", "plasma", "hash"] {
        assert!(
            !shader.contains(pattern),
            "the camera shell draws {pattern}"
        );
    }
}

/// The fog of the whiteout is nothing in clear air and grows with how much cloud the field holds
/// around the camera: twice the cloud, twice the fog.
#[test]
fn the_whiteout_fog_follows_the_cloud_around_the_camera() {
    assert_eq!(white(0.0), 0.0);
    let (thin, thick) = (white(0.25), white(0.5));
    assert!(thin > 0.0 && (thick / thin - 2.0).abs() < 1.0e-5);
    assert!(white(1.0) > white(0.9) && white(2.0) == white(1.0));
}
