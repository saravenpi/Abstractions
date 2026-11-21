//! The motion rig the runtime drives: one [`Motion`] per individual, stepped with a [`Drive`].
//!
//! Everything is in one local flat frame: +Y up, the ground a height function of `(x, z)`. The
//! rig keeps the root, the feet, the chains and the idle clocks of one body. [`Motion::pose`]
//! gives each part's place in that frame, so parts can be flat children of one root entity.

use bevy::math::{Quat, Vec2, Vec3};

use super::brace::Brace;
use super::feet::Foot;
use super::frame::chains;
use super::gait::Clock;
use super::idle::Idle;
use super::rest::Frame;
use super::root::yaw_of;
use super::song::Song;
use super::spring::Follow;
use super::strand::Strand;
use super::tune::Tune;
use super::turn::Turn;
use super::{plans, settle, walk};
use crate::life::body::{Body, Locomotion};
use crate::universe::hash::{Rng, child};

/// Salt of the seeded phase of the plan's cycle of an individual.
const CYCLE_KEY: u64 = 0xC1C1;

/// Salt of the seeded phase of the stride of an individual.
const STRIDE_KEY: u64 = 0x57D;

/// Salt of the seeded start of the rig's own clock of an individual.
const TIME_KEY: u64 = 0x71E;

/// Seconds after which the rig's own clock wraps, a whole number of turns of every timed wave.
const WRAP: f32 = 200.0 * std::f32::consts::PI;

/// How much work a body is worth this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Detail {
    /// Near: the ground under every foot, leg solvers and chains.
    Full,
    /// Far: gait phases with each foot on the ground under it and the body tilted to their plane, no
    /// leg solver over the steps.
    Cheap,
}

/// What the runtime asks of a body this step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drive {
    /// Desired velocity in the local frame, metres per second.
    pub velocity: Vec3,
    /// Point the head should look at, in the local frame.
    pub look: Option<Vec3>,
    /// Gravity of the planet in metres per second squared.
    pub gravity: f32,
    /// Alarm, from 0 calm to 1 alarmed.
    pub excite: f32,
    /// Level of detail.
    pub detail: Detail,
}

/// Where the body is and how it is held.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Root {
    /// The body origin: on the ground under the torso for walkers, the body itself for flyers.
    pub at: Vec3,
    /// Velocity, following the drive.
    pub vel: Follow<Vec3>,
    /// Heading in radians about +Y, 0 facing -Z.
    pub yaw: Follow<f32>,
    /// Height of the ground under the body, pitch and roll, held over the feet.
    pub hold: Follow<Vec3>,
    /// Height of the drawn ground under the root at the last step, which the hold rides on.
    pub floor: f32,
    /// Attitude of bodies that tilt into their way.
    pub tilt: Turn,
    /// Drawn place of the root this step.
    pub place: Vec3,
    /// Drawn rotation of the root this step.
    pub turn: Quat,
}

/// The clocks a body plan runs on its own and what they show.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Beat {
    /// Phase of the plan's cycle in turns: flap, pulse, body wave or hop.
    pub cycle: f32,
    /// Quarter turns a tumbler has rolled, 0 to 4.
    pub roll: f32,
    /// Angle of the wings in radians above the body plane.
    pub wing: f32,
    /// Scale of a squashing part, whose product stays 1.
    pub squash: Vec3,
    /// Seed of the individual as a share of a turn.
    pub seed: f32,
}

/// The motion of one individual.
#[derive(Debug, Clone)]
pub struct Motion {
    /// Root place, velocity and attitude.
    pub root: Root,
    /// Gait clock.
    pub clock: Clock,
    /// One foot per leg.
    pub feet: Vec<Foot>,
    /// One strand per chain: spine, neck, tail, then tentacles.
    pub strands: Vec<Strand>,
    /// Rest frame of the body.
    pub frame: Frame,
    /// Yaw and pitch of the head, following the look target.
    pub look: Follow<Vec2>,
    /// Idle clocks.
    pub idle: Idle,
    /// The plan's own clocks.
    pub beat: Beat,
    /// The call it is making, if any.
    pub song: Option<Song>,
    /// How it holds itself for an attack.
    pub brace: Brace,
    /// Tuning from the body's mass.
    pub tune: Tune,
    /// Detail of the last step, none before the first.
    pub detail: Option<Detail>,
    /// Seconds since the start, wrapped.
    pub time: f32,
}

impl Root {
    /// Returns a root standing still at `at` with heading `yaw`.
    fn new(at: Vec3, yaw: f32) -> Self {
        let turn = Quat::from_rotation_y(yaw);
        Self {
            at,
            vel: Follow::new(Vec3::ZERO, Vec3::ZERO),
            yaw: Follow::new(yaw, 0.0),
            hold: Follow::new(Vec3::new(at.y, 0.0, 0.0), Vec3::ZERO),
            floor: at.y,
            tilt: Turn::new(turn),
            place: at,
            turn,
        }
    }
}

impl Beat {
    /// Returns the clocks of the individual of `seed`, at rest, its cycle at a seeded phase so that a
    /// flock, a bloom or a herd spawned together never beats in step.
    fn new(seed: u64) -> Self {
        Self {
            cycle: Rng::new(child(seed, CYCLE_KEY)).unit() as f32,
            roll: 0.0,
            wing: 0.0,
            squash: Vec3::ONE,
            seed: (seed % 997) as f32 / 997.0,
        }
    }
}

impl Motion {
    /// Returns a body standing at `at` facing `facing`, both in the local frame, for the
    /// individual of `seed`.
    pub fn new(body: &Body, at: Vec3, facing: Vec3, seed: u64) -> Self {
        let root = Root::new(at, yaw_of(facing));
        let frame = Frame::of(body);
        let place = (at, root.turn);
        let feet = body.legs.iter().map(|l| Foot::new(at + place.1 * l.rest));
        let strands = chains(body).into_iter();
        let strands = strands.map(|(kind, c)| Strand::new(kind, frame.joints(c, place)));
        let clock = Clock {
            phase: Rng::new(child(seed, STRIDE_KEY)).unit() as f32,
            duty: 0.7,
            stride: body.height.max(0.05),
            ..Clock::default()
        };
        Self {
            root,
            clock,
            feet: feet.collect(),
            strands: strands.collect(),
            frame,
            look: Follow::new(Vec2::ZERO, Vec2::ZERO),
            idle: Idle::new(seed),
            beat: Beat::new(seed),
            song: None,
            brace: Brace::default(),
            tune: Tune::of_body(body),
            detail: None,
            time: Rng::new(child(seed, TIME_KEY)).unit() as f32 * WRAP,
        }
    }

    /// Moves the body `dt` seconds as `drive` asks, over the ground height `ground` of `(x, z)`.
    /// `body` must be the body the motion was made for.
    pub fn step(&mut self, body: &Body, drive: &Drive, ground: &impl Fn(Vec2) -> f32, dt: f32) {
        if dt.is_nan() || dt <= 0.0 {
            return;
        }
        let ground: &dyn Fn(Vec2) -> f32 = ground;
        self.time = (self.time + dt).rem_euclid(WRAP);
        self.idle
            .step(&self.tune, (drive.excite, self.work(body, drive)), dt);
        if let Some(song) = self.song.as_mut() {
            song.age += dt;
        }
        self.song = self.song.filter(Song::live);
        match body.locomotion {
            Locomotion::Walk => walk::step(self, body, drive, ground, dt),
            Locomotion::Slither => plans::slither(self, body, drive, ground, dt),
            Locomotion::Pulse => plans::pulse(self, body, drive, ground, dt),
            Locomotion::Ooze => plans::ooze(self, body, drive, ground, dt),
            Locomotion::Swarm => plans::swarm(self, body, drive, ground, dt),
            Locomotion::Tumble => plans::tumble(self, body, drive, ground, dt),
            Locomotion::Fly => plans::fly(self, body, drive, ground, dt),
            Locomotion::Swim => plans::swim(self, body, drive, ground, dt),
        }
        self.detail = Some(drive.detail);
        settle::attend(self, drive.look, dt);
        settle::bend(self, body, ground, dt);
    }

    /// Returns how hard the body of `body` works now under `drive`, 0 at rest to 1: its speed against a
    /// run, a Froude number of 1 on its legs, or the alarm it is in when that is more.
    fn work(&self, body: &Body, drive: &Drive) -> f32 {
        let run = (drive.gravity.max(0.1) * body.height.max(0.05)).sqrt();
        drive
            .excite
            .max(self.velocity().length() / run)
            .clamp(0.0, 1.0)
    }

    /// Returns the velocity of the body in the local frame.
    pub fn velocity(&self) -> Vec3 {
        self.root.vel.at
    }
}
