//! Warp optics against the references: the aberration and the Doppler factor the sky is drawn
//! with are the textbook ones, written in every form the references give, at many angles and
//! speeds (https://en.wikipedia.org/wiki/Relativistic_aberration,
//! https://en.wikipedia.org/wiki/Relativistic_Doppler_effect).

use bevy::math::DVec3;

use super::look::BETA_CAP;
use super::optics::{bend, doppler};
use super::shader::aberrate;

/// Speeds the references are checked at, as shares of the speed of light.
const SPEEDS: [f64; 5] = [0.05, 0.3, 0.6, 0.8, 0.85];

/// True angles from the heading, in radians, the references are checked at.
const ANGLES: [f64; 7] = [0.0, 0.2, 0.7, 1.2, 1.57, 2.4, 3.0];

/// Returns the Lorentz factor of the speed `beta`.
fn gamma(beta: f64) -> f64 {
    1.0 / (1.0 - beta * beta).sqrt()
}

/// The aberration is `cos θ' = (cos θ + β) / (1 + β cos θ)`, and also its tangent form
/// `tan θ' = sin θ / (γ (cos θ + β))`, in the rule of the shader and in the bend of the bodies.
#[test]
fn the_aberration_is_the_reference_one() {
    for beta in SPEEDS {
        for theta in ANGLES {
            let (sin, cos) = theta.sin_cos();
            let seen = ((cos + beta) / (1.0 + beta * cos)).clamp(-1.0, 1.0).acos();
            let tangent = sin.atan2(gamma(beta) * (cos + beta));
            assert!((seen - tangent).abs() < 1e-7, "{beta} {theta}");
            let shader = f64::from(aberrate(cos as f32, beta as f32)).acos();
            assert!(
                (shader - seen).abs() < 2e-3,
                "{beta} {theta}: {shader} {seen}"
            );
            let dir = DVec3::new(sin, 0.0, -cos);
            let bent = bend(dir, DVec3::NEG_Z, beta as f32).angle_between(DVec3::NEG_Z);
            assert!((bent - seen).abs() < 1e-6, "{beta} {theta}: {bent} {seen}");
        }
    }
    assert!(
        SPEEDS
            .iter()
            .any(|beta| (beta - f64::from(BETA_CAP)).abs() < 1e-6)
    );
}

/// The Doppler factor is `D = 1 / (γ (1 - β cos θ'))` with the seen angle, the same as
/// `γ (1 + β cos θ)` with the true one: `1 + z` of the light, inverted.
#[test]
fn the_doppler_factor_is_the_reference_one() {
    for beta in SPEEDS {
        let g = gamma(beta);
        for theta in ANGLES {
            let cos = theta.cos();
            let seen = (cos + beta) / (1.0 + beta * cos);
            let reference = 1.0 / (g * (1.0 - beta * seen));
            let boosted = g * (1.0 + beta * cos);
            assert!(
                (reference - boosted).abs() < 1e-9 * reference,
                "{beta} {theta}"
            );
            let drawn = f64::from(doppler(cos as f32, beta as f32));
            assert!(
                (drawn / reference - 1.0).abs() < 1e-5,
                "{beta} {theta}: {drawn}"
            );
        }
        let ahead = ((1.0 + beta) / (1.0 - beta)).sqrt();
        assert!((f64::from(doppler(1.0, beta as f32)) - ahead).abs() < 1e-4 * ahead);
        assert!((f64::from(doppler(0.0, beta as f32)) - g).abs() < 1e-5 * g);
    }
}

/// A small patch of sky is seen with its solid angle divided by `D²`: the aberration squeezes
/// it by `D` across and by `D` along, the rule the lens sizes the bodies with.
#[test]
fn a_patch_of_sky_shrinks_by_the_doppler_factor() {
    let (step, heading) = (1.0e-4, DVec3::NEG_Z);
    let at = |t: f64| DVec3::new(t.sin(), 0.0, -t.cos());
    for beta in SPEEDS {
        let seen = |t: f64| bend(at(t), heading, beta as f32);
        for theta in ANGLES[1..].iter().copied() {
            let along = seen(theta + step).angle_between(seen(theta)) / step;
            let across = seen(theta).angle_between(heading).sin() / theta.sin();
            let factor = f64::from(doppler(theta.cos() as f32, beta as f32));
            assert!(
                (along * factor - 1.0).abs() < 1e-3,
                "{beta} {theta}: {along}"
            );
            assert!(
                (across * factor - 1.0).abs() < 1e-3,
                "{beta} {theta}: {across}"
            );
        }
    }
}
