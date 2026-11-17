//! Filling the pose input from what the player controller already tells: standing, walking,
//! running, jumping, falling, landing, swimming, and sitting in the ship.

use bevy::prelude::*;

use super::clock::ramp;
use super::{AvatarPose, Mode};
use crate::player::{Motion, Player};
use crate::space::{Location, Travel};

/// Horizontal speed in meters per second above which a body on the ground walks.
pub const WALK_ABOVE: f32 = 0.3;

/// Horizontal speed in meters per second above which a body on the ground runs: between the
/// controller's walking and sprinting speeds.
pub const RUN_ABOVE: f32 = 6.5;

/// Fall speeds in meters per second between which a landing goes from soft to hard.
pub const IMPACT_SPAN: (f32, f32) = (9.0, 22.0);

/// Seconds the landing mode is held after touching down.
pub const LAND_SECONDS: f32 = 0.16;

/// What the filler remembers from one frame to the next.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Memory {
    /// Whether the body was off the ground and out of the water last frame.
    pub airborne: bool,
    /// Speed along the local up last frame, in meters per second.
    pub vertical: f32,
    /// Seconds of landing mode left.
    pub landing: f32,
}

/// Returns the mode of a body moving as `motion` says, `seated` in the ship or not.
pub fn classify(motion: &Motion, seated: bool) -> Mode {
    let speed = motion.velocity.with_y(0.0).length();
    match (seated, motion.swimming, motion.grounded) {
        (true, _, _) => Mode::Seated,
        (_, true, _) => Mode::Swim,
        (_, _, false) if motion.velocity.y > 0.5 => Mode::Jump,
        (_, _, false) => Mode::Fall,
        _ if speed > RUN_ABOVE => Mode::Run,
        _ if speed > WALK_ABOVE => Mode::Walk,
        _ => Mode::Idle,
    }
}

/// Returns how hard a landing at `fall` meters per second down is, from 0 to 1.
pub fn hardness(fall: f32) -> f32 {
    ramp(IMPACT_SPAN.0, IMPACT_SPAN.1, fall)
}

/// Writes one frame of `motion` into `pose`: the mode, the speeds and the distance walked. A body
/// that was in the air and now stands lands for a moment, as hard as it was falling.
pub fn observe(
    pose: &mut AvatarPose,
    memory: &mut Memory,
    motion: &Motion,
    (seated, dt): (bool, f32),
) {
    let mode = classify(motion, seated);
    let speed = motion.velocity.with_y(0.0).length();
    let standing = matches!(mode, Mode::Idle | Mode::Walk | Mode::Run);
    if standing && memory.airborne {
        pose.landed_hard = hardness(-memory.vertical);
        memory.landing = LAND_SECONDS;
    }
    let landing = standing && memory.landing > 0.0 && speed <= RUN_ABOVE;
    memory.landing = if standing { memory.landing - dt } else { 0.0 };
    memory.airborne = matches!(mode, Mode::Jump | Mode::Fall);
    memory.vertical = motion.velocity.y;
    pose.mode = if landing { Mode::Land } else { mode };
    pose.speed = speed;
    pose.vertical_speed = motion.velocity.y;
    pose.airspeed = motion.velocity.length();
    if standing {
        pose.stride_distance += speed * dt;
    }
}

/// Fills the pose input from the player controller. Systems ordered after
/// [`super::AvatarSet::Fill`] may overwrite any field, such as the flight modes and the gear.
pub fn fill(
    time: Res<Time>,
    location: Option<Res<Location>>,
    player: Single<&Motion, With<Player>>,
    mut pose: ResMut<AvatarPose>,
    mut memory: Local<Memory>,
) {
    let seated = location.is_some_and(|location| location.travel == Travel::InShip);
    observe(&mut pose, &mut memory, &player, (seated, time.delta_secs()));
}
