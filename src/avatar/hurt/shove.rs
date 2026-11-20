//! Shove: a blow on the robot pushes it and may stagger it. The impulse is the momentum of the blow
//! over the mass of the robot, added to its velocity through the controller, and a [`Reel`] leans the
//! body of the robot, laying it down when it loses its footing.

use bevy::math::{DQuat, DVec3, Vec2};
use bevy::prelude::*;

use super::super::blow::Blow;
use super::super::own::Own;
use super::super::strike::{Dir, Held, Strike};
use super::stagger::{Reel, absorb, capacity, guarded, impulse, reel};
use super::tune::GUARD_COS;
use crate::craft::build::mass;
use crate::fauna::senses::tune::ROBOT_KG;
use crate::player::{Motion, Player};
use crate::space::{Anchor, up};

/// Returns the share of a blow a guard takes when the body is guarding on `guard` and the blow, whose
/// direction in the frame of the body is `local`, comes at it from that side, holding a tool of
/// `tool` kilograms: the tool's mass over its mass and the arm's. Nothing when no guard faces the
/// blow or no tool is held.
pub fn blocked(guard: Option<Dir>, local: Vec3, tool: Option<f32>) -> Option<f32> {
    let from_right = -local.x;
    let faces = |side: f32| side != 0.0 && side * from_right > GUARD_COS;
    guard.filter(|g| faces(g.side()))?;
    tool.map(absorb)
}

/// Returns the push `dir` in the frame of the body as the right and ahead of it, for the body `placed`.
fn local_push(placed: &GlobalTransform, dir: Vec3) -> Vec3 {
    placed.rotation().inverse() * dir
}

/// Returns `dir`, a direction in render space, as a horizontal direction in the upright frame of a
/// body at `at`: the frame the controller keeps the velocity in.
pub fn upright(anchor: &Anchor, at: Vec3, dir: Vec3) -> Vec3 {
    let normal = up(anchor, at).as_dvec3().normalize();
    let frame = DQuat::from_rotation_arc(DVec3::Y, normal).as_quat();
    let local = frame.inverse() * dir;
    Vec3::new(local.x, 0.0, local.z).normalize_or_zero()
}

/// Shoves the local robot by every blow that landed on it: its velocity grows by the impulse along the
/// blow, cut by a guard on the side of the blow, and its body staggers or falls by the impulse over
/// what its stance absorbs.
pub fn shove(
    mut blows: MessageReader<Blow>,
    (anchor, own): (Res<Anchor>, Own),
    mut players: Query<(&Transform, &mut Motion), With<Player>>,
    bodies: Query<(&GlobalTransform, &Strike, Option<&Held>)>,
    mut commands: Commands,
) {
    let Some((player, body)) = own.body() else {
        return;
    };
    let (Ok((at, mut motion)), Ok((placed, strike, held))) =
        (players.get_mut(player), bodies.get(body))
    else {
        return;
    };
    for blow in blows.read() {
        let local = local_push(placed, blow.dir);
        let tool = held.map(|h| mass(&h.build.pieces()));
        let share = blocked(strike.guarding(), local, tool);
        let speed = guarded(impulse(blow.momentum, ROBOT_KG), share);
        let flat = upright(&anchor, at.translation, blow.dir);
        motion.velocity += flat * speed;
        let stance = capacity(motion.crouch, share.is_some());
        let push = Vec2::new(local.x, -local.z);
        if let Some(reel) = reel(speed, stance, push) {
            commands.entity(body).insert(reel);
        }
    }
}

/// Ages the stagger of every body and lets it go once it is over.
pub fn recover(time: Res<Time>, mut commands: Commands, mut reeling: Query<(Entity, &mut Reel)>) {
    for (body, mut reel) in &mut reeling {
        reel.age += time.delta_secs();
        if reel.age >= reel.time {
            commands.entity(body).remove::<Reel>();
        }
    }
}
