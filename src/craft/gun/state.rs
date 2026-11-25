//! Runtime firearm state, shot events, and projectile or flash components.

use bevy::prelude::*;

use super::GunKind;
use crate::craft::matter::Mineral;

/// Local player's active firearm state: cooldown, reload timer, ADS blend, recoil spring, and muzzle.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct GunState {
    /// Seconds remaining before the next shot can be fired.
    pub cooldown: f32,
    /// Seconds remaining on the current reload cycle.
    pub reloading: f32,
    /// Aim-down-sights blend from 0 (hipfire) to 1 (fully sighted).
    pub ads: f32,
    /// Current recoil rise and kickback displacement, unitless.
    pub recoil: f32,
    /// Current recoil velocity for spring-damper recovery.
    pub recoil_vel: f32,
    /// Seconds elapsed since the most recent muzzle flash.
    pub flash_age: f32,
    /// World-space muzzle tip position computed by the avatar gun placer.
    pub muzzle_world: Option<Vec3>,
}

/// Emitted whenever a firearm discharges a round.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ShotFired {
    /// Muzzle position in render space.
    pub origin: Vec3,
    /// Unit flight direction in render space.
    pub dir: Vec3,
    /// Chassis that fired the round.
    pub kind: GunKind,
    /// Barrel mineral of the firing gun.
    pub barrel: Mineral,
    /// Mineral of the discharged bullet.
    pub ammo: Mineral,
    /// True when fired by the local player.
    pub local: bool,
}

/// Flying projectile tracer stepped in render space until impact or expiry.
#[derive(Component, Debug, Clone, Copy)]
pub struct BulletTracer {
    /// Position at the start of the current step.
    pub prev: Vec3,
    /// Current tip position in render space.
    pub pos: Vec3,
    /// Velocity vector in metres per second.
    pub vel: Vec3,
    /// Seconds of flight remaining before despawn.
    pub life: f32,
    /// Base chassis damage carried by this round.
    pub damage: f32,
    /// Mineral the bullet was cast from.
    pub ammo: Mineral,
    /// Shooter entity excluded from solid raycasts, if known.
    pub shooter: Option<Entity>,
    /// True for rifle rounds, false for pistol rounds.
    pub rifle: bool,
    /// True when this client owns hit detection for this bullet.
    pub local: bool,
}

/// Brief point-light muzzle flash faded and despawned by [`super::ads::decay`].
#[derive(Component, Debug, Clone, Copy)]
pub struct MuzzleFlash {
    /// Seconds elapsed since spawn.
    pub age: f32,
}
