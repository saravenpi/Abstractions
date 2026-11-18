//! Flying a loop with the player: the companion keeps its wing all the way round, both ways.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::brain::Brain;
use super::motion::{Float, follow};
use super::spot::{Probe, Stance};
use crate::space::Anchor;
use crate::universe::BodyId;
use crate::world::Terrain;

/// Frame time of the simulated world.
const DT: f32 = 1.0 / 60.0;

/// Height of the bottom of the loop above the sea, in metres: well clear of any ground.
const ALOFT: f32 = 400.0;

/// Radius of the loop in metres and speed around it in metres per second: 0.8 rad/s, as fast
/// as the wings turn.
const LOOP: (f32, f32) = (50.0, 40.0);

/// Frames one loop takes.
const FRAMES: usize = (TAU * LOOP.0 / (LOOP.1 * DT)) as usize;

/// Returns where the player's eyes are and how fast they move on frame `frame` of a flight
/// toward -Z: level for the time of one loop, then a loop, upward when `sign` is 1 and downward
/// when it is -1, then level again.
fn path(frame: usize, sign: f32) -> (Vec3, Vec3) {
    let (radius, speed) = LOOP;
    let angle = (frame.saturating_sub(FRAMES) as f32 * speed * DT / radius).min(TAU);
    let level = frame as f32 * DT - angle * radius / speed;
    let centre = Vec3::new(0.0, ALOFT + radius * (1.0 + sign), -speed * level);
    let at = centre + Vec3::new(0.0, -sign * angle.cos(), -angle.sin()) * radius;
    (at, Vec3::new(0.0, sign * angle.sin(), -angle.cos()) * speed)
}

/// A companion following a player through the air.
struct Wingman {
    /// Its brain.
    brain: Brain,
    /// Its pose.
    at: Transform,
    /// Its motion.
    float: Float,
}

impl Wingman {
    /// Moves the player to `eye` at `velocity` for one frame, lets the companion follow, and
    /// returns the spot it aims for from the eyes of the player.
    fn step(&mut self, probe: &Probe, (eye, velocity): (Vec3, Vec3)) -> Vec3 {
        let stance = Stance {
            eye,
            forward: velocity.normalize(),
            up: Vec3::Y,
        };
        self.brain.observe(probe.anchor, stance, DT);
        self.brain.look = eye + stance.forward * 10.0;
        follow(probe, &self.brain, &mut self.at, &mut self.float, DT);
        self.brain.goal().0 - eye
    }
}

/// Flies level, a loop and level again with the `right` of the body known or not. Returns the
/// largest move in one frame of the spot the companion aims for against the player, in metres,
/// and how far to the left of the body the companion was at its least from the loop on.
fn looped(sign: f32, right: Vec3) -> (f32, f32) {
    let terrain = Terrain::new(3);
    let anchor = Anchor::surface(BodyId::default(), &terrain, Vec3::ZERO);
    let probe = Probe {
        anchor: &anchor,
        terrain: &terrain,
    };
    let mut bot = Wingman {
        brain: Brain::new(3),
        at: Transform::from_translation(path(0, sign).0),
        float: Float::default(),
    };
    (bot.brain.flight, bot.brain.side) = (1.0, right);
    let (mut jump, mut left) = (0.0_f32, f32::MAX);
    let mut last: Option<Vec3> = None;
    for frame in 0..3 * FRAMES {
        let spot = bot.step(&probe, path(frame, sign));
        jump = jump.max(last.replace(spot).map_or(0.0, |was| was.distance(spot)));
        let beside = (path(frame, sign).0 - bot.at.translation).dot(Vec3::X);
        left = left.min(if frame < FRAMES { f32::MAX } else { beside });
    }
    (jump, left)
}

/// Through a whole loop, up or down, the spot the companion aims for never jumps and the
/// companion stays on the left wing of the player. Without the body to tell the side, the spot
/// crosses over at the top, which is what the body is read for.
#[test]
fn keeps_its_wing_through_a_loop() {
    for sign in [1.0, -1.0] {
        let (jump, left) = looped(sign, Vec3::X);
        assert!(jump < 0.25, "turning {sign}: its spot jumps {jump} m");
        assert!(left > 1.0, "turning {sign}: it is {left} m to the left");
        let (lost, _) = looped(sign, Vec3::ZERO);
        assert!(lost > 2.0, "turning {sign}: the spot only moved {lost} m");
    }
}
