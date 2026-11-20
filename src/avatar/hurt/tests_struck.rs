//! Tests of where a blow lands and what it does to the condition of the robot.

use bevy::prelude::*;

use super::super::blow::Blow;
use super::super::pose::Pose;
use super::super::rig::{Seg, globals};
use super::plates::nearest;
use super::take::take;
use crate::condition::{Condition, Part};

/// Returns the part a point `at` of the segment `seg` lies on, for the body at rest.
fn at(seg: Seg, local: Vec3) -> Part {
    let joints = globals(&Pose::default());
    nearest(joints[seg as usize].transform_point(local), &joints).0
}

/// A point on each part of the body at rest lands on that part, and the back of the torso lands on
/// the wing that lies there.
#[test]
fn contact_points_pick_their_part() {
    assert_eq!(at(Seg::Head, Vec3::new(0.0, 0.19, -0.1)), Part::Head);
    assert_eq!(at(Seg::Torso, Vec3::new(0.0, 0.2, -0.1)), Part::Torso);
    assert_eq!(at(Seg::Torso, Vec3::new(0.12, 0.2, 0.15)), Part::WingR);
    assert_eq!(at(Seg::Torso, Vec3::new(-0.12, 0.2, 0.15)), Part::WingL);
    assert_eq!(at(Seg::ForearmL, Vec3::new(0.0, -0.1, 0.0)), Part::ArmL);
    assert_eq!(at(Seg::ShinR, Vec3::new(0.0, -0.2, 0.0)), Part::LegR);
    assert_eq!(at(Seg::BootL, Vec3::new(0.0, -0.045, -0.035)), Part::BootL);
}

/// A blow that lands on the robot lowers the health of the part it struck, and a cut hurts more.
#[test]
fn a_blow_hurts_the_part_it_struck() {
    let mut app = App::new();
    app.add_message::<Blow>()
        .init_resource::<Condition>()
        .add_systems(Update, take);
    for cut in [false, true] {
        app.world_mut().write_message(Blow {
            at: Vec3::ZERO,
            joules: 300.0,
            cut,
        });
        app.update();
    }
    let body = app.world().resource::<Condition>();
    assert!(body.of(Part::Torso) < 1.0);
    let blunt = 300.0 * crate::condition::tune::BLUNT / crate::condition::tune::PLATE[1];
    let cut = 300.0 * crate::condition::tune::CUT / crate::condition::tune::PLATE[1];
    assert!((body.of(Part::Torso) - (1.0 - blunt - cut)).abs() < 1e-4);
}
