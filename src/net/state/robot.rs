//! A robot on the wire and back: its heading against the local up of the frame, its pose
//! fields, and the pose blended between two records.

use abstractions_net::wire::{RobotBody, stride_delta};
use bevy::math::{DQuat, DVec3};

use super::store::Tracked;
use crate::avatar::{AvatarPose, Mode};
use abstractions_net::jitter::{Snap, View};

/// Returns the orientation of a body standing at `pos` in a frame centred on its body, turned
/// by nothing: frame +Y taken to the local up by the smallest rotation. Both ends of the wire
/// measure the heading against it, so the heading means the same in every render space.
pub fn upright(pos: DVec3) -> DQuat {
    DQuat::from_rotation_arc(DVec3::Y, pos.normalize_or(DVec3::Y))
}

/// Returns the heading about the local up of a body at `pos` with orientation `rot`, both in
/// a frame centred on its body.
pub fn heading(pos: DVec3, rot: DQuat) -> f32 {
    let ahead = upright(pos).inverse() * rot * DVec3::NEG_Z;
    (-ahead.x).atan2(-ahead.z) as f32
}

impl Tracked for RobotBody {
    /// Returns the snapshot of a robot record: its orientation is the heading turned about the
    /// local up.
    fn snap(self, time: f64) -> Snap<Self> {
        let rotation = upright(self.position) * DQuat::from_rotation_y(f64::from(self.yaw));
        Snap {
            time,
            position: self.position,
            velocity: Some(self.velocity),
            rotation: rotation.as_quat(),
            extra: self,
        }
    }
}

/// Returns the rank of `mode` in [`Mode::ALL`].
fn rank(mode: Mode) -> u8 {
    Mode::ALL.iter().position(|m| *m == mode).unwrap_or(0) as u8
}

/// Returns the mode of rank `rank` in [`Mode::ALL`], the last one past its end.
pub fn mode(rank: u8) -> Mode {
    Mode::ALL[usize::from(rank).min(Mode::ALL.len() - 1)]
}

/// Returns the body of a record for `pose`, placed at `pos` with velocity `vel` and heading
/// `yaw` in the frame of the record, whose hands do `swing` for `phase` seconds, as
/// [`super::swing::pack`] gives them.
pub fn body(
    pose: &AvatarPose,
    (pos, vel, yaw): (DVec3, DVec3, f32),
    (swing, phase): (u8, f32),
) -> RobotBody {
    RobotBody {
        position: pos,
        velocity: vel.as_vec3(),
        yaw,
        mode: rank(pose.mode),
        speed: pose.speed,
        vertical_speed: pose.vertical_speed,
        stride_distance: pose.stride_distance,
        bank: pose.bank,
        pitch: pose.pitch,
        thrust: pose.thrust,
        wings: pose.wings,
        airspeed: pose.airspeed,
        look: pose.look,
        landed_hard: pose.landed_hard,
        crouch: pose.crouch,
        swing,
        phase,
    }
}

/// The distance a remote body walked, ever growing as the pose wants it, from the wrapped
/// distance of its records.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Stride {
    /// The wrapped distance drawn last, or nothing before the first.
    last: Option<f32>,
    /// The distance walked since the body was first drawn.
    total: f32,
}

impl Stride {
    /// Moves on to the wrapped distance `now` and returns the distance walked so far.
    pub fn walk(&mut self, now: f32) -> f32 {
        let step = self.last.map_or(0.0, |last| stride_delta(last, now));
        self.last = Some(now);
        self.total += step;
        self.total
    }
}

/// Returns the pose of a remote body between its two records of `view`: numbers blended by
/// the weight of the view, the stride along the shortest way past the wrap, and the mode of
/// the newer record.
pub fn blend(view: &View<RobotBody>, stride: &mut Stride) -> AvatarPose {
    let (a, b, t) = (&view.older, &view.newer, view.blend);
    let mix = |x: f32, y: f32| x + (y - x) * t;
    let walked = a.stride_distance + stride_delta(a.stride_distance, b.stride_distance) * t;
    let mode = mode(b.mode);
    AvatarPose {
        mode,
        speed: mix(a.speed, b.speed),
        vertical_speed: mix(a.vertical_speed, b.vertical_speed),
        stride_distance: stride.walk(walked),
        bank: mix(a.bank, b.bank),
        pitch: mix(a.pitch, b.pitch),
        thrust: mix(a.thrust, b.thrust),
        wings: mix(a.wings, b.wings),
        airspeed: mix(a.airspeed, b.airspeed),
        look: a.look.lerp(b.look, t),
        landed_hard: mix(a.landed_hard, b.landed_hard),
        crouch: mix(a.crouch, b.crouch),
        prone: if mode.winged() {
            mix(a.wings, b.wings)
        } else {
            0.0
        },
        load: 1.0,
        ..AvatarPose::default()
    }
}
