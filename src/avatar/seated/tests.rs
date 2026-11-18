//! Tests of the pilot in the seat: the body fits the cockpit, its hands hold the real
//! controls and its head turns with the view.

use bevy::prelude::*;

use super::sit::piloting;
use super::{PALM, held, offset, reach, rest, seated};
use crate::avatar::Shown;
use crate::avatar::mesh::segment;
use crate::avatar::pose::Pose;
use crate::avatar::rig::{Seg, globals};
use crate::ship::SEAT;
use crate::ship::cockpit::{BACK, FLOOR, Levers, PAN};
use crate::view::{FACE, Heading};

/// Returns the controls at step `i` of a sweep through every corner of the stick and the lever.
fn swept(i: usize) -> Levers {
    let axis = |n: usize| (i / n % 3) as f32 - 1.0;
    Levers {
        stick: Vec3::new(axis(1), axis(3), axis(9)),
        throttle: (i / 27 % 3) as f32 * 0.5,
    }
}

/// Returns every vertex of the segments `segs` of a body in `pose`, in the axes of the hull.
fn in_hull(pose: &Pose, segs: &[Seg]) -> Vec<Vec3> {
    let frames = globals(pose);
    let root = SEAT.as_vec3() + offset();
    let of = |seg: &Seg| {
        let frame = frames[*seg as usize];
        segment(*seg)
            .points()
            .map(move |p| root + frame * p)
            .collect::<Vec<_>>()
    };
    segs.iter().flat_map(of).collect()
}

/// Returns where the palm of `side` of a body in `pose` is, in the frame of its root.
fn palm(pose: &Pose, side: usize) -> Vec3 {
    globals(pose)[Seg::HandL as usize + side] * (Vec3::NEG_Y * PALM)
}

/// Returns the lowest and the highest corner of the box around `points`.
fn bounds(points: &[Vec3]) -> (Vec3, Vec3) {
    let grow = |(low, high): (Vec3, Vec3), p: &Vec3| (low.min(*p), high.max(*p));
    points.iter().fold((Vec3::MAX, Vec3::MIN), grow)
}

/// The neck of the seated body is the pivot the first person view turns on, the soles are on
/// the floor of the tub, the hips on the pan, the back on the backrest, and the whole body is
/// inside the tub and under the glass wherever the controls are.
#[test]
fn the_pilot_sits_in_the_seat() {
    let neck = offset() + globals(&rest())[Seg::Head as usize].translation;
    assert!(neck.distance(-FACE) < 1.0e-5, "the neck is at {neck}");
    let boots = bounds(&in_hull(&rest(), &[Seg::BootL, Seg::BootR]));
    assert!(
        boots.0.y > FLOOR - 0.002 && boots.0.y < FLOOR + 0.02,
        "soles at {}",
        boots.0.y
    );
    let hips = bounds(&in_hull(&rest(), &[Seg::Pelvis]));
    assert!(
        hips.0.y > PAN - 0.002 && hips.0.y < PAN + 0.05,
        "hips at {}",
        hips.0.y
    );
    let trunk = bounds(&in_hull(&rest(), &[Seg::Pelvis, Seg::Torso, Seg::Head]));
    assert!(
        trunk.1.z < BACK + 0.002 && trunk.1.z > BACK - 0.04,
        "back at {}",
        trunk.1.z
    );
    for i in 0..81 {
        let (low, high) = bounds(&in_hull(&seated(held(&swept(i))), &Seg::ALL));
        assert!(low.x > -0.36 && high.x < 0.36, "{i}: {low} {high}");
        assert!(low.y > FLOOR - 0.002 && high.y < 1.08, "{i}: {low} {high}");
        assert!(low.z > -1.95 && high.z < BACK + 0.002, "{i}: {low} {high}");
    }
}

/// The left palm is on the handle of the throttle and the right palm on the grip of the
/// stick, wherever the real input puts them, and they move with them.
#[test]
fn the_hands_hold_the_real_controls() {
    let mut worst = 0.0_f32;
    for i in 0..81 {
        let grips = held(&swept(i));
        let pose = seated(grips);
        for (side, grip) in grips.into_iter().enumerate() {
            let [_, _, elbow] = reach(&rest(), side, grip);
            assert!(elbow >= 0.0 && pose.values().iter().all(|v| v.is_finite()));
            worst = worst.max(palm(&pose, side).distance(grip));
        }
    }
    assert!(worst < 0.015, "a palm is {worst} m off its grip");
    let at = |levers: Levers| {
        let pose = seated(held(&levers));
        [palm(&pose, 0), palm(&pose, 1)]
    };
    let pull = Levers {
        stick: Vec3::X,
        ..default()
    };
    let full = Levers {
        throttle: 1.0,
        ..default()
    };
    let (idle, pulled, pushed) = (at(Levers::default()), at(pull), at(full));
    assert!(pulled[1].z > idle[1].z + 0.04 && pulled[0].distance(idle[0]) < 1.0e-4);
    assert!(pushed[0].z < idle[0].z - 0.1 && pushed[1].distance(idle[1]) < 1.0e-4);
}

/// A grip that is out of reach is pointed at with a straight arm instead of breaking the pose.
#[test]
fn a_grip_out_of_reach_straightens_the_arm() {
    let far = Vec3::new(0.3, 0.2, -3.0);
    let [shoulder, out, elbow] = reach(&rest(), 1, far);
    assert!(elbow.abs() < 0.1 && shoulder.is_finite() && out.is_finite());
    let pose = seated([far, far]);
    let hand = palm(&pose, 1);
    let arm = globals(&pose)[Seg::UpperArmR as usize].translation;
    assert!((hand - arm).normalize().dot((far - arm).normalize()) > 0.999);
}

/// At the eyes the head follows the view and is hidden; from behind the ship the pilot looks
/// ahead and is drawn whole; nothing steps in between.
#[test]
fn the_head_turns_with_the_view_at_the_eyes() {
    let heading = Heading {
        yaw: 7.0,
        pitch: -0.4,
    };
    let (look, shown) = piloting(&heading, 0.0);
    assert!((look - Vec2::new(7.0 - std::f32::consts::TAU, -0.4)).length() < 1.0e-5);
    assert_eq!(shown, Shown::FirstPerson);
    assert_eq!(piloting(&heading, 1.0), (Vec2::ZERO, Shown::Full));
    let mut last = look;
    for step in 1..=100 {
        let (now, _) = piloting(&heading, step as f32 / 100.0);
        assert!(now.distance(last) < 0.02);
        last = now;
    }
}
