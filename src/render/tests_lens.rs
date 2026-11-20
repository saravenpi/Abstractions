//! Tests of the underwater pass: when it runs and what it fogs with.

use bevy::prelude::*;

use super::fog::UNDERWATER;
use super::murk::Murk;
use super::sky::linear;

/// The underwater pass runs once part of the lens can be under the surface, fogs in the linear
/// colour of the liquid at the documented visibility, and never runs on a dry planet.
#[test]
fn the_pass_runs_only_near_or_under_the_surface() {
    let pass = |height: f32| Murk::of(Some(UNDERWATER), Vec3::Y, height, 3000.0);
    assert_eq!(
        (pass(1.0).under, pass(0.05).under, pass(-4.0).under),
        (0.0, 1.0, 1.0)
    );
    assert!(!pass(0.05).submerged() && pass(-0.01).submerged());
    assert_eq!(pass(-1.0).colour, linear(UNDERWATER).truncate());
    assert!((pass(-1.0).density - 3.0 / 25.0).abs() < 1.0e-6);
    let dry = Murk::of(None, Vec3::Y, -2.0, 3000.0);
    assert_eq!((dry.under, dry.density), (0.0, 0.0));
}
