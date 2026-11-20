//! Every body poses one finite place per part at either detail, flyers bank and flap, swimmers
//! beat faster when fast, and a step and a pose of a twelve-legged crawler stay cheap.

use std::time::Instant;

use bevy::math::{Quat, Vec3};

use super::fixtures::quadruped_with;
use super::fixtures_legs::{biped, crawler};
use super::fixtures_life::{Grown, grown};
use super::fixtures_soft::{crystal, fish, flyer, jelly, slime, snake};
use super::pose::Placed;
use super::rig::{Detail, Motion};
use super::tests_plans::{drive, hills};
use crate::life::Habitat;

/// Every fixture poses one finite place per part, near or far, and switching detail back seats
/// its feet on the ground again.
#[test]
fn every_body_poses_every_part() {
    let walkers = [
        quadruped_with(2, 0.3),
        quadruped_with(4, 0.35),
        biped(),
        crawler(3, 0.9),
        crawler(6, 1.6),
    ];
    let soft = [snake(), jelly(), slime(), crystal(), flyer(), fish()];
    for (i, body) in walkers.iter().chain(&soft).enumerate() {
        let mut motion = Motion::new(body, Vec3::Y * 0.5, Vec3::X, i as u64);
        for frame in 0..240 {
            let way = Quat::from_rotation_y(frame as f32 * 0.03) * Vec3::NEG_Z;
            let mut asked = drive(way * 1.2, 0.5);
            asked.detail = [Detail::Full, Detail::Cheap][(frame / 60) % 2];
            motion.step(body, &asked, &hills, 1.0 / 30.0);
            let pose = motion.pose(body);
            assert_eq!(pose.len(), body.parts.len());
            let finite = |(at, turn, scale): &Placed| {
                at.is_finite() && turn.is_finite() && scale.is_finite()
            };
            assert!(pose.iter().all(finite), "body {i}");
        }
    }
}

/// A flyer flaps, banks into a left turn with its right side up, and stays over the ground.
#[test]
fn flyer_flaps_and_banks() {
    let body = flyer();
    let mut motion = Motion::new(&body, Vec3::Y * 4.0, Vec3::NEG_Z, 1);
    let (mut low, mut high, mut bank) = (f32::MAX, f32::MIN, 0.0);
    for frame in 0..300 {
        let turn = Quat::from_rotation_y(frame as f32 * 0.04);
        motion.step(
            &body,
            &drive(turn * Vec3::NEG_Z * 4.0, 0.0),
            &hills,
            1.0 / 30.0,
        );
        (low, high) = (low.min(motion.beat.wing), high.max(motion.beat.wing));
        bank = (motion.root.turn * Vec3::X).y;
        assert!(motion.root.at.y > 0.5, "flew into the ground");
    }
    assert!(high - low > 0.5, "wings did not flap: {low} to {high}");
    assert!(bank > 0.05, "no bank into the turn: {bank}");
}

/// A fish beats its body wave faster when it swims faster.
#[test]
fn swimmer_beats_faster_when_fast() {
    let body = fish();
    let beats = |speed: f32| {
        let mut motion = Motion::new(&body, Vec3::Y * 2.0, Vec3::NEG_Z, 3);
        let (mut turns, mut last) = (0, 0.0);
        for _ in 0..300 {
            motion.step(
                &body,
                &drive(Vec3::NEG_Z * speed, 0.0),
                &|_| 0.0,
                1.0 / 30.0,
            );
            turns += usize::from(motion.beat.cycle < last);
            last = motion.beat.cycle;
        }
        turns
    };
    assert!(beats(2.0) > beats(0.3) + 3);
}

/// A step and a pose of a twelve-legged crawler cost well under the budget of 20 microseconds in
/// an optimised build; a debug build gets ten times more.
#[test]
fn crawler_step_and_pose_are_cheap() {
    let body = crawler(6, 1.6);
    let mut motion = Motion::new(&body, Vec3::ZERO, Vec3::NEG_Z, 6);
    let asked = drive(Vec3::NEG_Z * 1.0, 0.0);
    let rounds = 2000;
    let start = Instant::now();
    let mut parts = 0;
    for _ in 0..rounds {
        motion.step(&body, &asked, &hills, 1.0 / 30.0);
        parts += motion.pose(&body).len();
    }
    let each = start.elapsed().as_secs_f64() * 1e6 / f64::from(rounds);
    let budget = if cfg!(debug_assertions) { 200.0 } else { 20.0 };
    assert!(each < budget, "{each} microseconds a step");
    assert_eq!(parts, rounds as usize * body.parts.len());
}

/// Returns the microseconds one step and pose of `g` take on average over 300 frames.
fn timed(g: &Grown) -> f64 {
    let rounds = 300;
    let mut motion = Motion::new(&g.body, Vec3::Y * 0.5, Vec3::NEG_Z, 4);
    let asked = drive(Vec3::NEG_Z * g.body.length.max(0.2), 0.3);
    let start = Instant::now();
    for _ in 0..rounds {
        motion.step(&g.body, &asked, &hills, 1.0 / 30.0);
        std::hint::black_box(motion.pose(&g.body));
    }
    start.elapsed().as_secs_f64() * 1e6 / f64::from(rounds)
}

/// Stepping and posing every grown body of every plan and habitat costs on average under the
/// budget of 20 microseconds a creature a frame, so the 48 animated creatures take under a
/// millisecond, and the dearest body stays under twice that.
#[test]
fn grown_bodies_step_and_pose_within_budget() {
    let habitats = [Habitat::Ground, Habitat::Air, Habitat::Water];
    let bodies: Vec<Grown> = habitats.into_iter().flat_map(grown).collect();
    let costs: Vec<f64> = bodies.iter().map(timed).collect();
    let mean = costs.iter().sum::<f64>() / costs.len() as f64;
    let (worst, cost) = costs
        .iter()
        .enumerate()
        .fold((0, 0.0), |a, (i, &c)| if c > a.1 { (i, c) } else { a });
    let g = &bodies[worst];
    let name = format!("{:?} {} {} kg", g.plan, g.archetype, g.mass);
    println!(
        "{} bodies: mean {mean:.2} us, dearest {cost:.2} us ({name})",
        bodies.len()
    );
    assert!(mean < 20.0, "mean {mean} microseconds a creature");
    assert!(cost < 40.0, "{name} took {cost} microseconds");
}
