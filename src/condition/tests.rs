//! Tests of the condition model.

use super::*;

/// A whole robot loses nothing to its effects.
#[test]
fn whole_robot_works_fully() {
    let body = Condition::default();
    assert_eq!(body.pace(), 1.0);
    assert_eq!(body.leap(), 1.0);
    assert_eq!(body.carry(), 1.0);
    assert_eq!(body.wings(), [1.0, 1.0]);
    assert_eq!(body.pull(), 0.0);
    assert!(body.damaged().is_empty());
}

/// A cut hurts more than a blunt blow of the same energy, and no part goes below zero.
#[test]
fn cut_bites_deeper_than_blunt() {
    let (mut cut, mut blunt) = (Condition::default(), Condition::default());
    let a = cut.hurt(Part::ArmL, 100.0, true);
    let b = blunt.hurt(Part::ArmL, 100.0, false);
    assert!(a > b);
    cut.hurt(Part::ArmL, 1.0e6, true);
    assert_eq!(cut.of(Part::ArmL), 0.0);
}

/// Both legs badly hurt make the robot crawl; one hurt leg only slows it.
#[test]
fn legs_limp_then_crawl() {
    let mut body = Condition::default();
    body.hurt(Part::LegL, 400.0, true);
    assert!(body.pace() < 1.0 && !body.crawling());
    body.hurt(Part::LegL, 1.0e6, true);
    body.hurt(Part::LegR, 1.0e6, true);
    assert!(body.crawling());
    assert!(body.pace() < 0.3);
}

/// A hurt right wing rolls the body to the right, and a broken arm holds nothing.
#[test]
fn wings_pull_and_arms_break() {
    let mut body = Condition::default();
    body.hurt(Part::WingR, 1.0e6, true);
    assert!(body.pull() > 0.0);
    body.hurt(Part::ArmL, 1.0e6, true);
    assert!(body.broken(0) && !body.broken(1));
}

/// Mending stops at the cap, and a lost head ruins the frame.
#[test]
fn mending_is_capped_and_head_ruins() {
    let mut body = Condition::default();
    body.hurt(Part::Torso, 1000.0, true);
    body.mend(Part::Torso, 1.0, 0.8);
    assert!((body.of(Part::Torso) - 0.8).abs() < 1e-6);
    body.hurt(Part::Head, 1.0e6, true);
    assert!(body.ruined());
}
