//! Tests of joint life, wear counting and explanations.

use super::rig::{earth, example, heavy};
use super::*;
use crate::craft::matter::{Class, Mineral};

/// A bound head lasts longer than a bare friction fit, and a missing counterweight never breaks.
#[test]
fn bindings_hold_heads() {
    let bound = example(Form::Edge, Mineral::Flint.material());
    let bare = Build {
        binding: None,
        ..bound
    };
    let (a, b) = (
        lives(&bound, &swing(&bound, &earth())),
        lives(&bare, &swing(&bare, &earth())),
    );
    assert!(a[Joint::Head as usize] > b[Joint::Head as usize]);
    assert!(a[Joint::Counterweight as usize] > 1.0e9);
}

/// Wear counts blows, breaks at the weakest joint, and a repair starts that joint again.
#[test]
fn builds_break_at_the_weakest_joint() {
    let lives = [500.0, f32::INFINITY, 3.0, 40.0];
    let mut wear = Wear::default();
    assert_eq!(wear.weakest(&lives), Joint::Edge);
    (0..3).for_each(|_| wear.hit());
    assert_eq!(wear.broken(&lives), Some(Joint::Edge));
    wear.repair(Joint::Edge);
    assert_eq!(wear.broken(&lives), None);
    assert_eq!(wear.weakest(&lives), Joint::Edge);
}

/// Obsidian chips on its first blows and says so; a hide edge is too soft for shell; a heavy head on
/// a heavy world is too heavy to swing.
#[test]
fn builds_explain_themselves() {
    let glass = example(Form::Edge, Mineral::Obsidian.material());
    let swung = swing(&glass, &earth());
    let life = lives(&glass, &swung);
    let done = strike(&blow(&glass, Intent::Cut, &earth()), &reference());
    let why = explain(&glass, &swung, &done, &reference(), &life);
    assert!(life[Joint::Edge as usize] < 1.0);
    assert!(why.contains(&Why::TooBrittle) && why.contains(&Why::WeakestJoint(Joint::Edge)));
    let soft = example(Form::Edge, Class::Hide.anchor());
    let done = strike(&blow(&soft, Intent::Cut, &earth()), &reference());
    let why = explain(&soft, &swung, &done, &reference(), &life);
    assert_eq!(why[0].to_string(), "edge too soft for shell");
    let maul = Build {
        head: Head {
            mass: 2.0,
            ..soft.head
        },
        ..soft
    };
    let there = swing(&maul, &heavy());
    let why = explain(&maul, &there, &done, &reference(), &life);
    assert!(why.contains(&Why::TooHeavy));
    assert_eq!(
        Why::TooHeavy.to_string(),
        "too heavy to swing on this world"
    );
}
