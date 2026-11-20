//! Springs settle on time, survive coarse steps, and lumber for heavy bodies.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};

use super::spring::{Follow, Spring};
use super::tune::Tune;
use super::turn::Turn;

/// Returns the largest error of a unit step response of `spring` after `settle` seconds, run for
/// `run` seconds at steps of `dt`.
fn worst_after(spring: &Spring, settle: f32, run: f32, dt: f32) -> f32 {
    let mut value = Follow::new(0.0f32, 0.0);
    let mut worst: f32 = 0.0;
    for i in 1..=(run / dt) as usize {
        value.step(spring, 1.0, dt);
        if i as f32 * dt >= settle {
            worst = worst.max((value.at - 1.0).abs());
        }
    }
    worst
}

/// An underdamped step response is within 2 percent of the target from 4 / (zeta 2 pi f) on.
#[test]
fn spring_settles_by_its_time() {
    for (f, zeta) in [(1.0, 0.6), (3.0, 0.6), (1.0, 0.8), (3.0, 0.8), (0.5, 0.6)] {
        let settle = 4.0 / (zeta * TAU * f);
        let worst = worst_after(
            &Spring::new(f, zeta, 0.0),
            settle,
            settle * 4.0,
            1.0 / 240.0,
        );
        assert!(
            worst < 0.02,
            "f {f} zeta {zeta}: off by {worst} after {settle} s"
        );
    }
}

/// A stiff, lightly damped spring stepped at 0.1 s neither blows up nor stays off target.
#[test]
fn spring_survives_coarse_steps() {
    for (f, zeta, r) in [(8.0, 0.35, 0.0), (6.0, 0.2, 2.0), (3.0, 1.0, -1.0)] {
        let spring = Spring::new(f, zeta, r);
        let mut value = Follow::new(Vec3::ZERO, Vec3::ZERO);
        for i in 0..400 {
            let target = if i % 100 < 50 {
                Vec3::X * 3.0
            } else {
                Vec3::NEG_Z
            };
            value.step(&spring, target, 0.1);
            assert!(
                value.at.is_finite() && value.at.length() < 30.0,
                "f {f}: {}",
                value.at
            );
        }
        assert!(
            value.at.distance(Vec3::NEG_Z) < 0.05,
            "f {f}: settled at {}",
            value.at
        );
    }
}

/// A rotation on a spring settles on its target, even stepped at 0.1 s.
#[test]
fn turn_settles_on_its_target() {
    let target = Quat::from_rotation_y(2.5) * Quat::from_rotation_x(0.7);
    for dt in [1.0 / 60.0, 0.1] {
        let mut turn = Turn::new(Quat::IDENTITY);
        for _ in 0..(8.0 / dt) as usize {
            turn.step(&Spring::new(2.0, 0.7, 0.0), target, dt);
        }
        assert!(
            turn.at.angle_between(target) < 0.01,
            "dt {dt}: {}",
            turn.at.angle_between(target)
        );
    }
}

/// Heavy bodies get slow, well damped springs and small ones quick, livelier ones.
#[test]
fn heavy_bodies_lumber() {
    let (mouse, cow) = (Tune::of_mass(0.03), Tune::of_mass(600.0));
    assert!(mouse.f > 3.0 * cow.f && cow.f < 1.2);
    assert!(cow.zeta > 0.8 && mouse.zeta < 0.5);
    assert!(mouse.breath() > cow.breath());
    assert!(Spring::new(cow.f, cow.zeta, 0.0) != Spring::new(mouse.f, mouse.zeta, 0.0));
}
