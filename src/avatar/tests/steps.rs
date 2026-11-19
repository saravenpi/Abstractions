//! A whole life of the body, frame by frame: every joint moves at a bounded rate through every
//! transition, at any frame rate. A snap is a step that does not shrink with the frame.
//!
//! The body is driven the way the game drives it: a small simulation moves a controller state
//! and a flight gear state, and the real filler and the real dresser turn them into the pose input.

use bevy::prelude::*;

use crate::avatar::animate::Anim;
use crate::avatar::dress::dress;
use crate::avatar::fill::{Memory, observe};
use crate::avatar::{AvatarPose, Mode};
use crate::gear::{FlightState, Mode as Flight};
use crate::player::Motion;

/// Fastest any joint may move, in radians per second (meters per second for the pelvis): a
/// sprinting leg does 27, and a change of mode adds its blend.
const RATE: f32 = 34.0;

/// Pull of gravity on the simulated body, in meters per second squared.
const GRAVITY: f32 = 22.0;

/// What the body is asked to do for a while.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Act {
    /// Reach this ground speed, in meters per second, turning at this rate in radians per second.
    Go(f32, f32),
    /// Jump, then fall back to the ground.
    Hop,
    /// Spread the wings and glide.
    Soar,
    /// Fire the boots on the wings and pull up.
    Burn,
    /// Fold the wings and come down on the boots.
    Flare,
    /// Swim at the surface, or under it.
    Swim(bool),
}

/// The life: seconds of each act, in order. It stands, walks, runs, jumps at a run and lands
/// in its stride, stops, jumps on the spot, jumps off into a glide, flies under power, flares
/// down on the boots, lands hard, swims, dives and stands again.
const LIFE: [(f32, Act); 17] = [
    (0.6, Act::Go(0.0, 0.0)),
    (1.0, Act::Go(5.0, 0.0)),
    (1.2, Act::Go(8.5, 0.8)),
    (1.1, Act::Hop),
    (0.6, Act::Go(8.5, -0.8)),
    (0.9, Act::Go(0.0, 0.0)),
    (1.2, Act::Hop),
    (0.5, Act::Go(3.0, 0.0)),
    (0.35, Act::Hop),
    (2.4, Act::Soar),
    (1.6, Act::Burn),
    (1.0, Act::Soar),
    (2.6, Act::Flare),
    (1.0, Act::Go(0.0, 0.0)),
    (1.2, Act::Swim(false)),
    (0.8, Act::Swim(true)),
    (1.0, Act::Go(0.0, 0.0)),
];

/// The simulated body: what the controller and the gear would hold.
#[derive(Default)]
struct Body {
    /// The controller state.
    motion: Motion,
    /// The gear state.
    gear: FlightState,
    /// Free height under the body, in meters.
    height: f32,
    /// Whether it is deep under water.
    deep: bool,
}

/// Returns `value` moved toward `goal` by at most `step`.
fn toward(value: f32, goal: f32, step: f32) -> f32 {
    value + (goal - value).clamp(-step, step)
}

impl Body {
    /// Moves the ground speed toward `speed` and turns the heading, on the feet.
    fn go(&mut self, (speed, turn): (f32, f32), dt: f32) {
        self.motion.swimming = false;
        self.deep = false;
        if !self.motion.grounded && self.gear.mode == Flight::Off {
            return self.fall(dt);
        }
        self.motion.grounded = true;
        self.motion.yaw += turn * dt;
        let ahead = toward(self.motion.velocity.z.abs(), speed, 25.0 * dt);
        self.motion.velocity = Vec3::new(0.0, 0.0, -ahead);
        self.gear.wings = toward(self.gear.wings, 0.0, dt / 0.4);
    }

    /// Falls freely and stands once the ground is reached.
    fn fall(&mut self, dt: f32) {
        self.motion.velocity.y -= GRAVITY * dt;
        self.height += self.motion.velocity.y * dt;
        if self.height <= 0.0 {
            (self.height, self.motion.grounded) = (0.0, true);
            self.motion.velocity.y = 0.0;
        }
    }

    /// Moves the gear toward a flight in `mode`: the wings and the boots spool at a steady
    /// rate, the attitude, the load and the speeds ease, as the real gear does.
    fn fly(&mut self, mode: Flight, (wings, thrust, pitch, load): (f32, f32, f32, f32), dt: f32) {
        let ease = 1.0 - (-4.0 * dt).exp();
        let gear = &mut self.gear;
        (gear.mode, self.motion.grounded) = (mode, false);
        gear.wings = toward(gear.wings, wings, dt / 0.4);
        gear.thrust = toward(gear.thrust, thrust, dt / 0.5);
        gear.prone += (wings - gear.prone) * ease;
        gear.pitch += (pitch - gear.pitch) * ease;
        gear.load += (load - gear.load) * ease;
        gear.bank += (0.5 * pitch.signum() - gear.bank) * ease * 0.5;
        gear.airspeed += (10.0 + 30.0 * wings - gear.airspeed) * ease;
        let sink = Vec3::new(0.0, -6.0 + 8.0 * pitch, -gear.airspeed);
        gear.velocity += (sink - gear.velocity) * ease;
        self.motion.velocity = gear.velocity;
    }

    /// Comes down on the boots from 8 m up and stands, the gear off, once the ground is reached.
    fn flare(&mut self, dt: f32) {
        if self.motion.grounded {
            self.gear.mode = Flight::Off;
            return self.go((0.0, 0.0), dt);
        }
        self.fly(Flight::BootFall, (0.0, 1.0, 0.0, 1.4), dt);
        self.height = (self.height.min(8.0) + self.gear.velocity.y * dt).max(0.0);
        self.motion.grounded = self.height <= 0.0;
    }

    /// Advances the body by `dt` seconds of `act`.
    fn act(&mut self, act: Act, dt: f32) {
        match act {
            Act::Go(speed, turn) => self.go((speed, turn), dt),
            Act::Hop => {
                if self.motion.grounded {
                    self.motion.grounded = false;
                    self.motion.velocity.y = 8.0;
                }
                self.fall(dt);
            }
            Act::Soar => self.fly(Flight::Glide, (1.0, 0.0, -0.3, 1.0), dt),
            Act::Burn => self.fly(Flight::Fly, (1.0, 1.0, 0.6, 2.8), dt),
            Act::Flare => self.flare(dt),
            Act::Swim(deep) => {
                (self.motion.swimming, self.motion.grounded) = (true, false);
                (self.deep, self.gear.mode) = (deep, Flight::Off);
                self.motion.velocity = Vec3::new(0.0, 0.0, -2.0);
            }
        }
        self.height = if self.gear.mode.winged() {
            60.0
        } else {
            self.height
        };
    }
}

/// The largest change of a joint in one frame, and where in the life it happened.
#[derive(Debug, Clone, Copy)]
struct Worst {
    /// The change, in radians or meters.
    step: f32,
    /// The act it happened in.
    act: Act,
    /// Index of the joint in the row of a pose.
    joint: usize,
}

/// Lives the whole life at `fps` frames per second and returns the largest change of a joint
/// in one frame, with where it happened, and the modes the life went through. The first
/// frames, in which the body takes its standing pose from nothing, do not count.
fn live(fps: f32) -> (Worst, Vec<Mode>) {
    let dt = 1.0 / fps;
    let (mut body, mut memory) = (Body::default(), Memory::default());
    let (mut input, mut anim) = (AvatarPose::default(), Anim::default());
    body.motion.grounded = true;
    let (mut worst, mut seen) = (None::<Worst>, Vec::new());
    for (seconds, act) in LIFE {
        for _ in 0..(seconds * fps) as usize {
            body.act(act, dt);
            observe(&mut input, &mut memory, &body.motion, (false, dt));
            dress(&mut input, &body.gear, Vec2::ZERO, body.deep);
            input.clearance = body.height;
            let before = anim.pose.values();
            let now = anim.step(&input, dt).values();
            let moved = before.iter().zip(now).map(|(a, b)| (a - b).abs());
            let (joint, step) = moved
                .enumerate()
                .fold((0, 0.0), |a, b| [a, b][usize::from(b.1 > a.1)]);
            let counts = anim.clock.time > 0.3 && worst.is_none_or(|worst| step > worst.step);
            worst = if counts {
                Some(Worst { step, act, joint })
            } else {
                worst
            };
            seen.extend((seen.last() != Some(&input.mode)).then_some(input.mode));
        }
    }
    (worst.expect("a life"), seen)
}

/// Through the whole life no joint ever moves faster than the bound, at 30 and at 144 frames
/// per second, and under the microscope of 1000 frames per second no step is left over: the
/// largest change in a frame shrinks with the frame, which a snap does not.
#[test]
fn no_joint_steps_through_a_whole_life() {
    for fps in [30.0, 144.0, 1000.0] {
        let (worst, seen) = live(fps);
        assert!(
            worst.step < RATE / fps,
            "at {fps} fps joint {} moves {} in a frame during {:?}",
            worst.joint,
            worst.step,
            worst.act
        );
        for mode in Mode::ALL {
            let skipped = mode == Mode::Seated;
            assert!(
                skipped || seen.contains(&mode),
                "at {fps} fps the life never passes {mode:?}: {seen:?}"
            );
        }
    }
}
