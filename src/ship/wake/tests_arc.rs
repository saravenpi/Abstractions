//! Ribbon tests: the trail of a tip is the path the tip flew, each tip and each lamp lays its
//! own, and the render anchor may move without moving any of them.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::lay::{Source, tip};
use crate::gear::{Look, Strip, Trail, trace};
use crate::space::{Anchor, Frame, Placed};
use crate::universe::BodyId;

/// Middle of the circle the hull of the turn tests flies, in the frame of the anchor, its
/// radius in metres and the bank of the hull in radians.
const TURN: (DVec3, f64, f64) = (DVec3::new(300.0, 1000.0, -200.0), 800.0, 0.6);

/// Returns the pose of a hull that flies a level circle at 120 m/s, banked into it, after `t`
/// seconds.
fn turning(t: f64) -> Placed {
    let (middle, radius, bank) = TURN;
    let turn = 120.0 / radius * t;
    Placed {
        pos: middle + DVec3::new(turn.cos(), 0.0, turn.sin()) * radius,
        rot: DQuat::from_rotation_y(std::f64::consts::PI - turn) * DQuat::from_rotation_z(-bank),
    }
}

/// Returns the trail `source` lays over five seconds of the turn at 60 frames a second.
fn laid(source: Source) -> Trail {
    let mut trail = Trail::default();
    for frame in 0..300 {
        let placed = turning(f64::from(frame) / 60.0);
        trail.feed_by(tip(&placed, source), 1.0, 1.0 / 60.0, (2.5, 60.0));
    }
    trail
}

/// After five seconds of a constant banked turn the points of each trail lie on the circle its
/// emitter flew, within 2 cm, at the height of the emitter, and they are the last 60 points of
/// that path: the ribbon is where the tip was, not a straight stick. The two wing tips and the
/// two lamps each lay their own, on four different circles.
#[test]
fn a_trail_lies_on_the_arc_its_tip_flew() {
    let sources = [0, 1]
        .map(Source::Vortex)
        .into_iter()
        .chain([0, 1].map(Source::Lamp));
    let mut radii = Vec::new();
    for source in sources {
        let (trail, now) = (laid(source), tip(&turning(299.0 / 60.0), source));
        let from_axis = |p: DVec3| (p - TURN.0).with_y(0.0).length();
        assert!(
            trail.len() > 60 && trail.tip().at.distance(now) < 1.0e-6,
            "{source:?}"
        );
        for index in 0..trail.len() {
            let point = trail.point(index).at;
            let off = (from_axis(point) - from_axis(now))
                .abs()
                .max((point.y - now.y).abs());
            assert!(
                off < 0.02,
                "{source:?}: point {index} is {off} m off its arc"
            );
            assert!(
                point.distance(now) < 2.5 * (index as f64 + 2.0),
                "{source:?} {index}"
            );
        }
        radii.push(from_axis(now));
    }
    let apart = |a: usize, b: usize| (radii[a] - radii[b]).abs() > 3.0;
    assert!(apart(0, 1) && apart(2, 3), "{radii:?}");
}

/// Returns the frame positions of the head and of the middle of rows 5, 20 and 40 of the ribbon
/// of `trail`, drawn under `anchor`.
fn drawn(trail: &Trail, anchor: &Anchor) -> Vec<DVec3> {
    let look = Look {
        eye: Vec3::new(3.0, 40.0, 9.0),
        width: 0.06,
        spread: 5.0,
        life: 10.0,
        tint: Vec3::ONE,
    };
    let (mut strip, mut shown) = (Strip::default(), false);
    let head = trace(trail, &mut shown, (look, anchor), &mut strip).expect("a live trail");
    let rows = [5, 20, 40].map(|row| Vec3::from_array(strip.positions[row * 3 + 1]));
    let all = [Vec3::ZERO].into_iter().chain(rows);
    all.map(|at| anchor.to_frame(head + at)).collect()
}

/// A trail does not move when the render anchor does: drawn under an anchor at the ship and
/// under one 400 m away and turned, its ribbon is at the same places of the frame within 5 mm.
#[test]
fn an_anchor_shift_leaves_a_trail_where_it_is() {
    let trail = laid(Source::Vortex(0));
    let frame = Frame::Surface(BodyId::default());
    let here = Anchor {
        frame,
        origin: turning(5.0).pos,
        rotation: DQuat::IDENTITY,
        flat: false,
    };
    let moved = Anchor {
        origin: here.origin + DVec3::new(250.0, -30.0, 310.0),
        rotation: DQuat::from_rotation_y(0.7) * DQuat::from_rotation_x(0.2),
        ..here
    };
    for (a, b) in drawn(&trail, &here).into_iter().zip(drawn(&trail, &moved)) {
        assert!(a.distance(b) < 0.005, "{a} and {b}");
    }
}
