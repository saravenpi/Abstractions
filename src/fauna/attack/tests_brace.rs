//! Tests of the body in the wind-up: the hips drop more for a bigger commitment, the weight goes back
//! and is thrown forward, and everything is at ease again after the recovery.

use bevy::math::{Quat, Vec3};

use super::brace::{brace, phases};
use super::kinds::Kind;
use super::timing::Attacking;
use super::tune::RECOVER;
use crate::fauna::motion::brace::Brace;

/// Returns an attack of `kind` that has run for `t` seconds on a body of one metre under 10 m/s².
fn at(kind: Kind, t: f32) -> Attacking {
    Attacking {
        t,
        ..Attacking::begin(kind, 1.0, 10.0)
    }
}

/// The hips drop by a share that grows with the commitment: a charge more than a bite, and at the end
/// of the wind-up more than at its start.
#[test]
fn the_hips_drop_with_the_commitment() {
    let wind = at(Kind::Bite, 0.0).wind;
    let drop = |kind: Kind, t: f32| brace(&at(kind, t), 1.0).drop;
    assert!(drop(Kind::Charge, wind) > drop(Kind::Trample, wind));
    assert!(drop(Kind::Trample, wind) > drop(Kind::Lunge, wind));
    assert!(drop(Kind::Lunge, wind) > drop(Kind::Bite, wind));
    assert!(drop(Kind::Charge, wind) > drop(Kind::Charge, 0.5 * wind));
    assert_eq!(drop(Kind::Charge, 0.0), 0.0);
}

/// The weight goes back in the wind-up, is thrown forward in the blow and the recovery, and the body
/// is back at ease once the recovery is over.
#[test]
fn the_weight_goes_back_then_forward_then_home() {
    let a = at(Kind::Charge, 0.0);
    let (wind, blow) = (a.wind, a.blow());
    assert!(brace(&at(Kind::Charge, wind), 1.0).shift > 0.0);
    assert!(brace(&at(Kind::Charge, wind + blow), 1.0).shift < 0.0);
    let off = brace(&at(Kind::Charge, wind + blow + 0.2 * RECOVER * wind), 1.0);
    assert!(off.shift < 0.0 && off.pitch < 0.0);
    let end = at(Kind::Charge, wind + blow + RECOVER * wind + 0.01);
    assert_eq!(brace(&end, 1.0), Brace::default());
}

/// A trample rears, a charge ducks, a sweep turns its flank and swings back through.
#[test]
fn each_attack_holds_its_own_pose() {
    let wind = at(Kind::Bite, 0.0).wind;
    assert!(brace(&at(Kind::Trample, wind), 1.0).pitch > 0.0);
    assert!(brace(&at(Kind::Charge, wind), 1.0).pitch < 0.0);
    assert!(brace(&at(Kind::Sweep, wind), 1.0).yaw > 0.0);
    assert!(brace(&at(Kind::Sweep, wind + at(Kind::Sweep, 0.0).blow()), 1.0).yaw < 0.0);
    assert_eq!(brace(&at(Kind::Bite, wind), 1.0).yaw, 0.0);
}

/// The phases are continuous and stay within 0 and 1.
#[test]
fn the_phases_stay_within_range() {
    let (wind, blow) = (0.7, 0.18);
    let mut last = (0.0, 0.0);
    for step in 0..400 {
        let t = step as f32 * 0.005;
        let (load, thrown) = phases(t, wind, blow);
        assert!((0.0..=1.0).contains(&load) && (0.0..=1.0).contains(&thrown));
        assert!((load - last.0).abs() < 0.1 && (thrown - last.1).abs() < 0.1);
        last = (load, thrown);
    }
}

/// A brace at rest leaves the root alone, and a drop lowers it while the hips stay the pivot of a
/// pitch.
#[test]
fn a_brace_lowers_the_body_about_the_hips() {
    let root = (Vec3::new(1.0, 0.8, 0.0), Quat::IDENTITY);
    assert_eq!(Brace::default().placed(root, 1.0), root);
    let low = Brace {
        drop: 0.2,
        ..Brace::default()
    };
    assert!((low.placed(root, 1.0).0.y - 0.6).abs() < 1e-6);
    let rear = Brace {
        pitch: 0.5,
        ..Brace::default()
    };
    let (place, turn) = rear.placed(root, 1.0);
    let hips = place + turn * Vec3::Y;
    assert!((hips - (root.0 + Vec3::Y)).length() < 1e-5);
}
