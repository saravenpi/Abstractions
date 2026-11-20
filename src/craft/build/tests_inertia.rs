//! Tests of mass properties and swings, on the plan's worked example.

use super::rig::{earth, example, heavy};
use super::*;
use crate::craft::matter::{Class, Mineral};

/// Returns true when `a` lies within 5 % of `b`.
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.05 * b
}

/// 0.3 kg × 0.6 m shaft and a 0.8 kg head: I ≈ 0.32 kg·m², ω ≈ 24 rad/s, ≈ 93 J.
#[test]
fn worked_example_matches_the_plan() {
    let pieces = [
        Piece::rod(0.3, 0.0, 0.6),
        Piece {
            mass: 0.8,
            at: 0.6,
            gyration: 0.0,
        },
    ];
    let i = inertia(&pieces);
    let spin = tip_speed(i) / 0.6;
    assert!(near(i, 0.32), "I {i}");
    assert!(near(spin, 24.0), "ω {spin}");
    assert!(
        near(0.5 * i * spin * spin, 93.0),
        "E {}",
        0.5 * i * spin * spin
    );
    let swung = swing(&example(Form::Mass, Mineral::Basalt.material()), &earth());
    assert!(
        near(swung.inertia, 0.32) && near(swung.energy, 93.0),
        "{swung:?}"
    );
}

/// Hold torque g·Σm·r is 5.6 N·m on 9.81 m/s² and 12.5 N·m on 22: one hand holds both, but the swing
/// that fits the budget at home stretches past it with a heavier head on the heavy world.
#[test]
fn gravity_sets_the_hold_torque() {
    let build = example(Form::Edge, Mineral::Flint.material());
    let (home, there) = (swing(&build, &earth()), swing(&build, &heavy()));
    assert!(near(home.torque, 9.81 * 0.57) && near(there.torque, 22.0 * 0.57));
    assert!(there.torque < Hands::One.budget() && home.windup == there.windup);
    let maul = Build {
        head: Head {
            mass: 1.2,
            ..build.head
        },
        shaft: Shaft {
            length: 0.8,
            ..build.shaft
        },
        ..build
    };
    let (light, burden) = (swing(&maul, &earth()), swing(&maul, &heavy()));
    assert!(light.torque < Hands::One.budget() && burden.torque > Hands::One.budget());
    assert!(burden.windup > light.windup);
}

/// A counterweight at the butt pulls the balance back and lowers the hold torque; a heavier build
/// thrusts slower.
#[test]
fn counterweight_and_mass_move_the_numbers() {
    let build = example(Form::Point, Class::Bone.anchor());
    let weighted = Build {
        grip: 0.12,
        counterweight: Some(Weight {
            mass: 0.4,
            material: Mineral::Basalt.material(),
        }),
        ..build
    };
    assert!(balance(&weighted.pieces()) < balance(&build.pieces()));
    assert!(swing(&weighted, &earth()).torque < swing(&build, &earth()).torque);
    let heavier = Build {
        head: Head {
            mass: 1.6,
            ..build.head
        },
        ..build
    };
    assert!(swing(&heavier, &earth()).thrust < swing(&build, &earth()).thrust);
}
