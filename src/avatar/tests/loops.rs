//! The body through a whole loop on the wings: it follows the flight path all the way round, both ways,
//! without a jump, and ends upright.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::FRAME;
use super::flown::{Flown, JOINT_STEP, LEAD, TURN, head, looped};
use crate::avatar::Mode;
use crate::avatar::air::level;
use crate::avatar::pose::{CORE_LIMITS, Core};

/// Share of its pitch the frame of a gear with folded wings keeps from one frame to the next.
const SETTLE: f32 = 0.98;

/// On the wings the body follows the flight path all the way round, up and down, gliding and
/// under power: no joint and no segment jumps, half way round it lies on its back with its head
/// still turned to the view, and at the end it lies as it started.
#[test]
fn the_body_follows_a_whole_loop_both_ways() {
    let cases = [Mode::Glide, Mode::Fly].map(|mode| [(mode, 1.0), (mode, -1.0)]);
    for (mode, sign) in cases.into_iter().flatten() {
        let what = format!("{mode:?} turning {sign}");
        let start = Flown::level(mode).anim.pose;
        let (flown, top) = looped(mode, sign);
        let joint = flown.worst.x;
        assert!(joint < JOINT_STEP, "{what}: a joint jumps by {joint}");
        flown.assert_smooth(&what);
        let (ahead, back) = (head(&start), head(&top));
        assert!(ahead.x > 0.4 && back.x < -0.4, "{what}: {ahead}, {back}");
        let turned = flown.anim.pose.core(Core::Pitch) - start.core(Core::Pitch);
        assert!((turned + sign * TAU).abs() < 1e-3, "{what}: {turned}");
        let end = head(&flown.anim.pose);
        assert!(end.distance(ahead) < 0.05, "{what}: ends at {end}");
        let nod = top.core(Core::HeadPitch) - start.core(Core::HeadPitch);
        assert!((nod - sign * LEAD).abs() < 0.02, "{what}: nods {nod}");
    }
}

/// Folds the wings of a body and returns how far its trunk swings in all while it rights itself.
///
/// The view stays where it was, and the gear turns its frame level the short way, as it does:
/// to the whole turn nearest the pitch it had.
fn righted(flown: &mut Flown) -> f32 {
    flown.worst = Vec3::ZERO;
    let (from, view) = (flown.pitch, flown.pitch + flown.lead);
    let rest = (from / TAU).round() * TAU;
    let mut swept = 0.0;
    let mut last = head(&flown.anim.pose).to_angle();
    for frame in 1..=60 {
        let pitch = rest + (from - rest) * SETTLE.powi(frame);
        flown.lead = view - pitch;
        let now = head(&flown.step(Mode::Fall, pitch)).to_angle();
        swept += (now - last + PI).rem_euclid(TAU) - PI;
        last = now;
    }
    swept
}

/// After a loop the gear still counts the turn it made. Folding the wings then, or about half
/// way round on its back, rights the body the short way: nothing jumps, the trunk never swings
/// through a second turn, and it ends upright inside the limits of a body on its feet.
#[test]
fn the_body_ends_upright_after_a_loop() {
    for (sign, turns) in [(1.0, 1.0), (-1.0, 1.0), (1.0, 0.45), (-1.0, 0.55)] {
        let (mut flown, _) = looped(Mode::Glide, sign);
        let frames = (turns * TAU / (TURN * FRAME)) as usize;
        for frame in 0..frames {
            flown.step(Mode::Glide, sign * (TAU + TURN * FRAME * frame as f32));
        }
        let swept = righted(&mut flown);
        let what = format!("{turns} turns {sign}");
        flown.assert_smooth(&what);
        println!(
            "{what}: the trunk swings {swept} rad, worst {:?}",
            flown.worst
        );
        assert!(swept.abs() < PI + 0.2, "{what}: the trunk swings {swept}");
        let lean = flown.anim.pose.core(Core::Pitch);
        let (low, high) = CORE_LIMITS[Core::Pitch as usize];
        assert!((low..=high).contains(&lean), "{what}: ends leaning {lean}");
        let up = head(&flown.anim.pose);
        assert!(up.y > 0.4 && up.x.abs() < 0.2, "{what}: head at {up}");
        assert!(flown.anim.path.abs() <= PI, "{what}: a turn is left");
    }
}

/// A view that passed the vertical in a loop reads as where it points: the arms and the head
/// follow the half turn around the horizon, and an ordinary view is left exactly as it is.
#[test]
fn a_view_on_its_back_reads_as_where_it_points() {
    for pitch in [-1.5, -0.2, 0.0, 0.45, 1.553, 3.0] {
        assert_eq!(level(pitch), pitch);
    }
    assert!((level(TAU + 0.2) - 0.2).abs() < 1e-5);
    assert!((level(-TAU - 0.2) + 0.2).abs() < 1e-5);
    assert!((level(PI + 0.5) + PI - 0.5).abs() < 1e-5);
}
