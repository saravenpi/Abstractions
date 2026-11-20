//! Tests of the beat of an attack, the fall of a body that died and the attacks bodies have.

use super::kinds::Kind;
use super::limbs::Broken;
use super::slump::{plunge, tip};
use super::timing::{Attacking, joules, wind};
use super::tune::{LEAN, STRIKE};
use crate::fauna::rig::planet;

/// The wind-up grows with the body: 0.68 s for a metre under 22 m/s², 1.1 s for four.
#[test]
fn the_wind_up_grows_with_size() {
    assert!((wind(1.0, 22.0) - 0.676).abs() < 0.01);
    assert!((wind(4.0, 22.0) - 1.1).abs() < 0.03);
    assert!(wind(0.1, 22.0) < wind(1.0, 22.0));
    assert!(wind(1.0, 5.0) > wind(1.0, 22.0));
}

/// An attack is drawn back through its wind-up, thrown out in its blow and eased back to rest.
#[test]
fn the_swing_goes_back_then_out_then_home() {
    let mut a = Attacking::begin(Kind::Bite, 1.0, 22.0);
    assert_eq!(a.swing(), 0.0);
    a.t = a.wind;
    assert!((a.swing() + 1.0).abs() < 1e-4);
    a.t = a.wind + 0.5 * STRIKE;
    assert!(a.striking() && a.swing().abs() < 1e-3);
    a.t = a.wind + STRIKE;
    assert!((a.swing() - 1.0).abs() < 1e-3 && !a.striking());
    a.t += 1.5 * a.wind;
    assert!(a.over() && a.swing() < 1e-3);
}

/// A body that died tips over to lie at its rest angle, still, in a few seconds, and a flyer lands.
#[test]
fn a_body_that_died_falls_and_lies_still() {
    let rest = std::f32::consts::FRAC_PI_2;
    let (mut angle, mut rate, mut done) = (LEAN.0, LEAN.1, false);
    for _ in 0..600 {
        (angle, rate, done) = tip((angle, rate), (1.2, 9.8, rest), 1.0 / 60.0);
        if done {
            break;
        }
    }
    assert!(done && (angle - rest).abs() < 1e-3);
    let mut air = (6.0, 0.0, 0.0);
    for _ in 0..600 {
        air = plunge(air, 9.8, 1.0 / 60.0);
    }
    assert!(air.0 == 0.0 && (air.1 - 6.0).abs() < 1e-3);
}

/// A charge or a trample carries more energy than a bite of the same body.
#[test]
fn a_dash_hits_harder_than_a_bite() {
    assert!(joules(Kind::Charge, 200.0, 8.0) > joules(Kind::Bite, 200.0, 8.0));
    assert!(joules(Kind::Trample, 600.0, 5.0) > joules(Kind::Swipe, 600.0, 5.0));
}

/// Bodies with horns charge, those with jaws bite, heavy ones trample, and nothing has an attack
/// whose parts are all broken.
#[test]
fn bodies_have_the_attacks_their_parts_give() {
    let (_, _, life) = planet(true);
    let life = life.expect("the home planet lives");
    let mut kinds = 0;
    for species in &life.species {
        for kind in Kind::ORDER {
            if kind.can(species, Broken::default()) {
                kinds += 1;
                assert!(kind.reach(&species.body) > 0.0);
            }
        }
        let every = Broken(u64::MAX);
        assert!(!Kind::Charge.can(species, every) && !Kind::Sweep.can(species, every));
        assert!(!Kind::Bite.can(species, every));
    }
    assert!(kinds > 0);
}
