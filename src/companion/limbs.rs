//! The hands and the thruster: hands trail behind the body and take part in the idles, the thruster pulses.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::Companion;
use super::motion::Float;
use super::pose::Pose;
use super::puppet::Parts;
use super::tune::{HALF, HAND_LAG};

/// How fast a hand catches up with where it should trail, per second.
const HAND_RATE: f32 = 7.0;

/// Where a hand rests beside the body: out to its side, below the middle, slightly forward, in meters.
const HAND_REST: Vec3 = Vec3::new(HALF[0] + 0.065, -0.05, -0.03);

/// Size of the thruster glow as a scale of the shared orb.
pub const THRUSTER: Vec3 = Vec3::new(2.6, 1.2, 2.6);

/// A floating hand of the companion.
#[derive(Component, Debug, Clone, Copy)]
pub struct Hand {
    /// Which side it floats on: 1 for the body's +X, -1 for its -X.
    pub side: f32,
    /// How far it currently trails behind its resting place, in the body's axes.
    pub lag: Vec3,
}

/// Marks the glowing thruster under the body.
#[derive(Component, Debug, Clone, Copy)]
pub struct Thruster;

/// The parts on a rig the limbs move: their transform, the hand if one is, and whether it is the thruster.
type Limbs<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        Option<&'static mut Hand>,
        Has<Thruster>,
    ),
    Without<Companion>,
>;

/// Returns where a hand trails to for a body moving at `local` meters per second in its own axes.
pub fn trail(local: Vec3) -> Vec3 {
    (-local * HAND_LAG.0).clamp_length_max(HAND_LAG.1)
}

/// Returns the transform of the hand on `side` at `now` seconds, trailing by `lag` and posed by `pose`.
///
/// At rest the hands sway slowly and out of step. A spread pose stretches them out and up, a
/// waving pose lifts them in turn.
pub fn hand_pose(side: f32, lag: Vec3, pose: &Pose, now: f32) -> Transform {
    let sway = 0.014 * (now * 1.7 + side * 1.3).sin();
    let wave = pose.wave * 0.07 * (now * 8.0 + side * FRAC_PI_2).sin();
    let reach = Vec3::new(
        side * pose.spread * 0.11,
        pose.spread * 0.13 + sway + wave,
        0.0,
    );
    let rest = HAND_REST * Vec3::new(side, 1.0, 1.0);
    Transform::from_translation(rest + reach + lag)
        .with_rotation(Quat::from_rotation_z(side * (0.25 + pose.spread * 0.9)))
}

/// Moves the hands of one companion moving at `float` and posed by it, and pulses its thruster.
fn swing(
    limb: (Mut<Transform>, Option<Mut<Hand>>, bool),
    local: Vec3,
    float: &Float,
    clock: (f32, f32),
) {
    let (mut transform, hand, thruster) = limb;
    let (now, blend) = clock;
    if let Some(mut hand) = hand {
        let lag = hand.lag.lerp(trail(local), blend);
        hand.lag = lag;
        *transform = hand_pose(hand.side, lag, &float.pose, now).with_scale(transform.scale);
    } else if thruster {
        let push = 1.0 + 0.12 * (now * 11.0).sin() + (float.vel.length() * 0.08).min(0.5);
        transform.scale = THRUSTER * Vec3::new(1.0, push, 1.0);
    }
}

/// Moves the hands and pulses the thruster of every companion, each from its own motion and
/// pose, so a companion driven from elsewhere moves its own hands too.
pub fn limbs(
    time: Res<Time>,
    bots: Query<(&Transform, &Float, &Parts), With<Companion>>,
    rigs: Query<&Children>,
    mut parts: Limbs,
) {
    let (now, dt) = (time.elapsed_secs_wrapped(), time.delta_secs());
    let blend = 1.0 - (-HAND_RATE * dt).exp();
    for (at, float, own) in &bots {
        let local = at.rotation.inverse() * float.vel;
        let Ok(children) = rigs.get(own.rig) else {
            continue;
        };
        for &child in children {
            if let Ok(limb) = parts.get_mut(child) {
                swing(limb, local, float, (now, blend));
            }
        }
    }
}
