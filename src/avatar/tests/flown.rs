//! A body flown by hand through a loop: the gear's pitch is set frame by frame and every move of the body
//! is measured.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{FRAME, gap};
use crate::avatar::air::level;
use crate::avatar::animate::Anim;
use crate::avatar::blend::Weights;
use crate::avatar::dress::dress;
use crate::avatar::pose::Pose;
use crate::avatar::rig::{Seg, globals};
use crate::avatar::{AvatarPose, Mode};
use crate::gear::{FlightState, Mode as Flight};

/// Fastest turn of the flight path the wings give, in radians per second.
pub const TURN: f32 = 0.9;

/// How far the view leads the nose in a held loop, in radians.
pub const LEAD: f32 = 0.3;

/// Largest change of a joint in one frame of a loop, in radians: the turn of the path in a frame
/// and the tremble of the limbs.
pub const JOINT_STEP: f32 = 0.05;

/// Largest move of a segment in one frame, in metres, and its largest turn in radians, in a
/// loop and while the body rights itself after it.
const BODY_STEP: (f32, f32) = (0.08, 0.16);

/// Returns the gear of a body in the flight of `mode` whose flight frame is at `pitch`.
fn gear(mode: Mode, pitch: f32) -> FlightState {
    let flight = match mode {
        Mode::Glide => Flight::Glide,
        Mode::Fly => Flight::Fly,
        _ => Flight::Fall,
    };
    FlightState {
        mode: flight,
        pitch,
        airspeed: 40.0,
        wings: 1.0,
        prone: 1.0,
        ..FlightState::default()
    }
}

/// Returns the largest move in metres and the largest turn in radians of a segment of the body
/// between two poses.
fn moved(before: &Pose, now: &Pose) -> Vec2 {
    let (was, is) = (globals(before), globals(now));
    let parts = was.iter().zip(is.iter()).map(|(a, b)| {
        let turn = a.rotation.angle_between(b.rotation);
        Vec2::new(a.translation.distance(b.translation), turn)
    });
    parts.fold(Vec2::ZERO, Vec2::max)
}

/// A body flown by a gear whose pitch is set by hand.
pub struct Flown {
    /// The animation of the body.
    pub anim: Anim,
    /// What the dresser wrote last.
    input: AvatarPose,
    /// The largest change of a joint, move of a segment and turn of a segment in one frame.
    pub worst: Vec3,
    /// The pitch of the gear on the last frame, on the turn the gear has reached.
    pub pitch: f32,
    /// How far the view is above the pitch of the gear, in radians.
    pub lead: f32,
}

impl Flown {
    /// Returns a body settled in level flight in `mode`.
    pub fn level(mode: Mode) -> Self {
        let mut flown = Self {
            anim: Anim {
                weights: Weights::only(mode),
                ..default()
            },
            input: AvatarPose::default(),
            worst: Vec3::ZERO,
            pitch: 0.0,
            lead: 0.0,
        };
        for _ in 0..90 {
            flown.step(mode, 0.0);
        }
        flown.worst = Vec3::ZERO;
        flown
    }

    /// Runs one frame with the gear in the flight of `mode` at `pitch`, dressed as in the game
    /// with the view ahead of the gear by the lead, and records how far the body moved.
    pub fn step(&mut self, mode: Mode, pitch: f32) -> Pose {
        self.pitch = pitch;
        let look = Vec2::Y * level(pitch + self.lead);
        dress(&mut self.input, &gear(mode, pitch), look, false);
        let before = self.anim.pose;
        let now = self.anim.step(&self.input, FRAME);
        let far = moved(&before, &now);
        self.worst = self.worst.max(Vec3::new(gap(&before, &now), far.x, far.y));
        now
    }

    /// Asserts that no segment of the body jumped or snapped round since the worst was reset.
    pub fn assert_smooth(&self, what: &str) {
        let (jump, snap) = (self.worst.y, self.worst.z);
        assert!(jump < BODY_STEP.0, "{what}: a segment jumps {jump} m");
        assert!(snap < BODY_STEP.1, "{what}: a segment snaps {snap} rad");
    }
}

/// Returns how far the head is ahead of the hips and above them, in metres.
pub fn head(pose: &Pose) -> Vec2 {
    let at = globals(pose).map(|frame| frame.translation);
    let offset = at[Seg::Head as usize] - at[Seg::Pelvis as usize];
    Vec2::new(-offset.z, offset.y)
}

/// Flies a whole loop in `mode`, `sign` being 1 for a pull up and -1 for a push down, the view
/// coming to lead the nose as a held key makes it, and returns the body at the end of it with
/// the pose it had half way round.
pub fn looped(mode: Mode, sign: f32) -> (Flown, Pose) {
    let mut flown = Flown::level(mode);
    let frames = (TAU / (TURN * FRAME)).ceil() as usize;
    let mut top = Pose::default();
    for frame in 1..=frames {
        let pitch = sign * TAU * frame as f32 / frames as f32;
        flown.lead = sign * LEAD * (frame as f32 / 20.0).min(1.0);
        let pose = flown.step(mode, pitch);
        if frame == frames / 2 {
            top = pose;
        }
    }
    (flown, top)
}
