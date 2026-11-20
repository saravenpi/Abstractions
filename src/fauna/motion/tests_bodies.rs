//! Grown bodies at walk, trot and gallop: the drawn end of every planted leg stays put through its
//! stance, and flyers and swimmers run their cycles without a jump.

use bevy::math::{Quat, Vec2, Vec3};

use super::fixtures_life::{G, Grown, grown};
use super::rig::{Detail, Drive, Motion};
use super::tests_plans::hills;
use crate::life::Habitat;
use crate::life::body::{Body, Locomotion};

/// Froude numbers of a walk, a trot and a gallop, inside each band of the gait table.
const GAITS: [(&str, f32); 3] = [("walk", 0.25), ("trot", 1.4), ("gallop", 3.6)];

/// Largest slip in metres of a drawn foot through one stance.
const SLIP: f32 = 0.02;

/// Returns a drive at `velocity` on the home-like world.
fn drive(velocity: Vec3) -> Drive {
    Drive {
        velocity,
        look: None,
        gravity: G,
        excite: 0.0,
        detail: Detail::Full,
    }
}

/// Returns the drawn end of every leg of `body` posed by `motion`.
fn tips(body: &Body, motion: &Motion) -> Vec<Vec3> {
    let pose = motion.pose(body);
    let tip = |leg: &crate::life::body::Leg| {
        let (bone, length) = (leg.bones.last(), leg.lengths.last());
        let (Some(&bone), Some(&length)) = (bone, length) else {
            return Vec3::ZERO;
        };
        let (at, turn, _) = pose[usize::from(bone)];
        at + turn * (motion.frame.axes[usize::from(bone)] * length)
    };
    body.legs.iter().map(tip).collect()
}

/// Walks `body` 8 s on a slow curve over the hills at Froude number `fr`, and returns the largest
/// slip of a drawn foot through one stance, counted from the frame after it lands.
fn slip(body: &Body, fr: f32) -> f32 {
    let speed = (fr * G * body.height.max(0.05)).sqrt();
    let start = Vec3::new(0.0, hills(Vec2::ZERO), 0.0);
    let mut motion = Motion::new(body, start, Vec3::NEG_Z, 9);
    let mut landed: Vec<Option<Vec3>> = vec![None; body.legs.len()];
    let mut worst: f32 = 0.0;
    for frame in 0..240 {
        let way = Quat::from_rotation_y(0.004 * frame as f32) * Vec3::NEG_Z;
        let before: Vec<bool> = motion.feet.iter().map(|f| f.planted).collect();
        motion.step(body, &drive(way * speed), &hills, 1.0 / 30.0);
        let tips = tips(body, &motion);
        for (i, foot) in motion.feet.iter().enumerate() {
            landed[i] = match (foot.planted, landed[i]) {
                (false, _) => None,
                (true, None) if !before[i] => Some(tips[i]),
                (true, held) => held,
            };
            if let Some(at) = landed[i].filter(|_| frame > 30) {
                worst = worst.max(at.distance(tips[i]));
            }
        }
    }
    worst
}

/// Every legged grown body keeps the drawn end of a planted leg within a few centimetres through
/// its whole stance at a walk, a trot and a gallop.
#[test]
fn grown_walkers_do_not_slip_at_any_gait() {
    let walkers: Vec<Grown> = grown(Habitat::Ground)
        .into_iter()
        .filter(|g| g.body.locomotion == Locomotion::Walk && !g.body.legs.is_empty())
        .collect();
    assert!(walkers.len() >= 6, "only {} walkers grown", walkers.len());
    let mut failed = Vec::new();
    for g in &walkers {
        let slips: Vec<f32> = GAITS.iter().map(|&(_, fr)| slip(&g.body, fr)).collect();
        println!(
            "{:?} {} {} kg, {} legs: walk {:.4} trot {:.4} gallop {:.4} m",
            g.plan,
            g.archetype,
            g.mass,
            g.body.legs.len(),
            slips[0],
            slips[1],
            slips[2]
        );
        let bad = GAITS.iter().zip(&slips).filter(|(_, s)| **s >= SLIP);
        failed
            .extend(bad.map(|(&(gait, _), s)| format!("{:?} {} {gait} {s}", g.plan, g.archetype)));
    }
    assert!(failed.is_empty(), "slipped: {failed:?}");
}

/// Returns the largest move of any part relative to the root in one frame, over body lengths, and
/// the largest advance of the plan's cycle in one frame, of `body` cruising 8 s at `speed`.
fn jumps(body: &Body, speed: f32) -> (f32, f32) {
    let mut motion = Motion::new(body, Vec3::Y * 4.0, Vec3::NEG_Z, 2);
    let (mut part, mut cycle): (f32, f32) = (0.0, 0.0);
    let mut last: Option<(Vec<Vec3>, f32)> = None;
    for frame in 0..240 {
        let way = Quat::from_rotation_y(0.01 * frame as f32) * Vec3::NEG_Z;
        motion.step(body, &drive(way * speed), &|_| 0.0, 1.0 / 30.0);
        let rel: Vec<Vec3> = motion
            .pose(body)
            .iter()
            .map(|p| p.0 - motion.root.place)
            .collect();
        if let Some((was, beat)) = &last {
            let moved = was.iter().zip(&rel).map(|(a, b)| a.distance(*b));
            part = part.max(moved.fold(0.0, f32::max) / body.length.max(0.05));
            cycle = cycle.max((motion.beat.cycle - beat).rem_euclid(1.0));
        }
        last = Some((rel, motion.beat.cycle));
    }
    (part, cycle)
}

/// Every grown flyer and swimmer moves no part more than a fifth of its length against the root in
/// one frame, and its cycle advances under 0.4 turns a frame, so no beat aliases.
#[test]
fn grown_flyers_and_swimmers_cycle_smoothly() {
    let mut bodies = grown(Habitat::Air);
    bodies.extend(grown(Habitat::Water));
    let movers: Vec<Grown> = bodies
        .into_iter()
        .filter(|g| matches!(g.body.locomotion, Locomotion::Fly | Locomotion::Swim))
        .collect();
    assert!(movers.iter().any(|g| g.body.locomotion == Locomotion::Fly));
    assert!(movers.iter().any(|g| g.body.locomotion == Locomotion::Swim));
    for g in &movers {
        let speed = 1.5 * g.body.length.max(0.1);
        let (part, cycle) = jumps(&g.body, speed);
        println!(
            "{:?} {:?} {} {} kg: part {part:.3} lengths, cycle {cycle:.3} turns a frame",
            g.body.locomotion, g.plan, g.archetype, g.mass
        );
        assert!(
            part < 0.2,
            "{:?} {}: part jumped {part}",
            g.plan,
            g.archetype
        );
        assert!(
            cycle < 0.4,
            "{:?} {}: cycle stepped {cycle}",
            g.plan,
            g.archetype
        );
    }
}
