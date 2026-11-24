//! Follower steering and lifecycle hooks for creatures driven by remote snapshots.

use std::collections::HashMap;

use bevy::math::{DVec3, Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::*;

use super::beast::{Beast, Part};
use super::motion::root::{wrap, yaw_of};
use super::motion::spring::Follow;
use super::motion::{Drive, Motion};
use super::place::Spot;
use super::senses::Stance;
use super::steer::{Pace, cruise};
use super::tune::{DRIFT, REACH};
use super::want::{self, From, Lead, Want};
use crate::life::body::{Body, Locomotion};
use crate::life::herd::Ledger;
use crate::life::hunt::Carcass;
use crate::life::{Life, Species};
use crate::space::Placed;
use crate::world::{Chunks, Terrain};

/// Maximum forward extrapolation window (seconds) beyond a snapshot timestamp.
const LEAD_MAX: f64 = 0.20;

/// Position error (metres) above which a follower snaps cleanly instead of steering.
const SNAP_DIST: f32 = 6.0;

/// Planet-frame error (metres) above which a snapped follower also refreshes its tangent spot.
const RESITE_DIST: f32 = 20.0;

/// Proportional tracking gain (`1 / s`) added on top of snapshot velocity feedforward.
const TRACK_GAIN: f32 = 5.0;

/// Error (metres) below which a nearly stopped follower rests without creeping.
const STILL_GAP: f32 = 0.08;

/// Speed (metres per second) below which a stationary creature aligns toward its synced heading.
const STILL_VEL: f32 = 0.05;

/// Authoritative remote snapshot of one living creature on the current planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyncTarget {
    /// Species index on the planet.
    pub species: u16,
    /// Planet-frame position in metres.
    pub pos: DVec3,
    /// Planet-frame velocity in metres per second.
    pub vel: Vec3,
    /// Planet-frame unit forward direction.
    pub heading: Vec3,
    /// Current stance replicated from the owner.
    pub stance: Stance,
    /// Current alarm / excitement in `0.0..=1.0`.
    pub excite: f32,
    /// Universe time at which the snapshot was sampled.
    pub time: f64,
}

/// Creatures currently steered by another client on this planet, keyed by `Lead::key()`.
#[derive(Resource, Debug, Default)]
pub struct RemoteFauna {
    /// Active authoritative targets keyed by `(herd_or_carcass, slot)`.
    pub targets: HashMap<(u64, u8), SyncTarget>,
}

impl RemoteFauna {
    /// Returns the active snapshot for creature key `(herd, slot)`, if remotely owned.
    pub fn get(&self, key: (u64, u8)) -> Option<&SyncTarget> {
        self.targets.get(&key)
    }

    /// Returns true when creature key `(herd, slot)` is currently remotely owned.
    pub fn contains(&self, key: (u64, u8)) -> bool {
        self.targets.contains_key(&key)
    }
}

/// Mutable query of child animal part transforms posed after each motion step.
pub(super) type Parts<'w, 's> = Query<
    'w,
    's,
    &'static mut Transform,
    (
        With<Part>,
        Without<crate::player::Player>,
        Without<crate::avatar::Avatar>,
    ),
>;

/// Returns true when `locomotion` places the creature's root on the surface.
fn grounded(locomotion: Locomotion) -> bool {
    matches!(
        locomotion,
        Locomotion::Walk | Locomotion::Slither | Locomotion::Ooze | Locomotion::Tumble
    )
}

/// Re-anchors `beast` onto a fresh tangent frame under where it currently stands.
pub(super) fn refit(
    terrain: &Terrain,
    (beast, placed): (&mut Beast, &mut Placed),
    species: &Species,
) {
    let radius = terrain.globe().radius();
    let old = beast.spot;
    let here = old.planet(beast.motion.root.at);
    let ahead = old.rot * (beast.motion.root.turn * Vec3::NEG_Z).as_dvec3();
    let spot = Spot::at(here.normalize(), radius);
    let facing = (spot.rot.inverse() * ahead)
        .as_vec3()
        .with_y(0.0)
        .normalize_or(Vec3::NEG_Z);
    let aloft = matches!(species.body.locomotion, Locomotion::Fly | Locomotion::Swarm);
    let lift = if aloft { cruise(&species.body) } else { 0.0 };
    let at = Vec3::new(0.0, spot.ground(terrain, Vec2::ZERO) + lift, 0.0);
    let seed = beast.lead.key().0 ^ u64::from(beast.lead.key().1);
    beast.motion = Motion::new(&species.body, at, facing, seed);
    beast.mood.threat = spot.local(old.planet(beast.mood.threat));
    beast.spot = spot;
    placed.pos = spot.origin;
    placed.rot = spot.rot;
}

/// Injects or updates living remotely owned creatures near `from.robot` in `wants`.
pub(super) fn fold_wants(
    remote: &RemoteFauna,
    (life, carcasses, ledger): (&Life, &[Carcass], &Ledger),
    from: From,
    wants: &mut Vec<Want>,
) {
    if remote.targets.is_empty() {
        return;
    }
    for (&(herd, slot), target) in &remote.targets {
        if ledger.dead(herd, slot, from.time)
            || !life
                .species
                .get(usize::from(target.species))
                .is_some_and(want::drawn)
        {
            continue;
        }
        let dir = target.pos.normalize();
        let metres = super::herds::herd_arc(dir, from.robot) * from.radius;
        if metres > REACH {
            continue;
        }
        if let Some(existing) = wants.iter_mut().find(|w| w.lead.key() == (herd, slot)) {
            existing.dir = dir;
            existing.metres = existing.metres.min(metres);
        } else {
            let lead = carcasses
                .iter()
                .find(|c| want::carcass_key(c) == herd)
                .map_or(Lead::Herd { id: herd, slot }, |c| Lead::Visit {
                    key: herd,
                    slot,
                    at: c.at,
                });
            wants.push(Want {
                lead,
                species: target.species,
                dir,
                metres,
            });
        }
    }
    wants.sort_unstable_by(|a, b| {
        a.metres
            .total_cmp(&b.metres)
            .then(a.lead.key().cmp(&b.lead.key()))
    });
}

/// Computes the initial tangent spot, local position and facing for a freshly spawned follower.
pub(super) fn birth_pose(
    target: &SyncTarget,
    terrain: &Terrain,
    body: &Body,
) -> (Spot, Vec3, Vec3) {
    let spot = Spot::at(target.pos.normalize(), terrain.globe().radius());
    let local = spot.local(target.pos);
    let at = if grounded(body.locomotion) {
        Vec3::new(local.x, spot.ground(terrain, local.xz()), local.z)
    } else {
        local
    };
    let facing = (spot.rot.inverse() * target.heading.as_dvec3())
        .as_vec3()
        .with_y(0.0)
        .normalize_or(Vec3::NEG_Z);
    (spot, at, facing)
}

/// Resets `beast`'s motion rig cleanly at `goal_planet` when a follower is too far off target.
fn snap_follower(
    terrain: &Terrain,
    (beast, placed): (&mut Beast, &mut Placed),
    body: &Body,
    (goal_planet, vel_planet, fwd_planet): (DVec3, DVec3, DVec3),
    respot: bool,
) {
    if respot {
        let spot = Spot::at(goal_planet.normalize(), terrain.globe().radius());
        beast.spot = spot;
        placed.pos = spot.origin;
        placed.rot = spot.rot;
    }
    let spot = beast.spot;
    let local = spot.local(goal_planet);
    let at = if grounded(body.locomotion) {
        Vec3::new(local.x, spot.ground(terrain, local.xz()), local.z)
    } else {
        local
    };
    let facing = (spot.rot.inverse() * fwd_planet)
        .as_vec3()
        .with_y(0.0)
        .normalize_or(Vec3::NEG_Z);
    let vel = (spot.rot.inverse() * vel_planet).as_vec3();
    let seed = beast.lead.key().0 ^ u64::from(beast.lead.key().1);
    beast.motion = Motion::new(body, at, facing, seed);
    beast.motion.root.vel.at = if grounded(body.locomotion) {
        vel.with_y(0.0)
    } else {
        vel
    };
}

/// Drives a follower creature's procedural rig toward `target` and poses its child parts.
pub(super) fn follow(
    (terrain, chunks, heights): (&Terrain, Option<&Chunks>, &super::ground::Heights),
    (beast, placed): (&mut Beast, &mut Placed),
    (species, parts): (&Species, &mut Parts),
    (target, time, dt): (&SyncTarget, f64, f32),
    pace: Pace,
) {
    let lead = (time - target.time).clamp(0.0, LEAD_MAX);
    let (goal_planet, vel_planet, fwd_planet) = (
        target.pos + target.vel.as_dvec3() * lead,
        target.vel.as_dvec3(),
        target.heading.as_dvec3(),
    );
    beast.wits.stance = target.stance;
    beast.mood.alarm = target.excite;
    let is_ground = grounded(species.body.locomotion);
    let goal_local = beast.spot.local(goal_planet);
    let err = goal_local - beast.motion.root.at;
    let gap = if is_ground {
        err.xz().length()
    } else {
        err.length()
    };
    let out_of_spot = goal_local.xz().length() > DRIFT;
    if out_of_spot || gap > SNAP_DIST {
        let respot = out_of_spot || gap > RESITE_DIST;
        snap_follower(
            terrain,
            (beast, placed),
            &species.body,
            (goal_planet, vel_planet, fwd_planet),
            respot,
        );
    } else {
        let spot = beast.spot;
        let vel_local = (spot.rot.inverse() * vel_planet).as_vec3();
        let fwd_local = (spot.rot.inverse() * fwd_planet)
            .as_vec3()
            .with_y(0.0)
            .normalize_or(Vec3::NEG_Z);
        let vel_mag = if is_ground {
            vel_local.xz().length()
        } else {
            vel_local.length()
        };
        let desired = if gap < STILL_GAP && vel_mag < STILL_VEL {
            Vec3::ZERO
        } else {
            let combined = vel_local + err * TRACK_GAIN;
            let raw = if is_ground {
                combined.with_y(0.0)
            } else {
                combined
            };
            raw.clamp_length_max((pace.top * 2.0).max(8.0))
        };
        let drive = Drive {
            velocity: desired,
            look: Some(beast.motion.root.at + fwd_local * 4.0),
            gravity: pace.gravity,
            excite: target.excite,
            detail: pace.detail,
        };
        let slant = spot.slant(terrain, beast.motion.root.at.xz());
        let drawn = |x: f32, z: f32| heights.drawn(terrain, chunks, x, z);
        let ground = |p: Vec2| spot.ground_near(terrain, (&slant, p), drawn);
        beast.motion.step(&species.body, &drive, &ground, dt);
        if desired.xz().length() <= STILL_VEL && gap <= STILL_GAP {
            let cur = beast.motion.root.yaw.at;
            let next = cur + wrap(yaw_of(fwd_local) - cur) * (1.0 - (-6.0 * dt).exp());
            beast.motion.root.yaw = Follow::new(next, 0.0);
        }
    }
    for (part, (at, turn, scale)) in beast.parts.iter().zip(beast.motion.pose(&species.body)) {
        if let Some(mut t) = part.and_then(|p| parts.get_mut(p).ok()) {
            t.set_if_neq(
                Transform::from_translation(at)
                    .with_rotation(turn)
                    .with_scale(scale),
            );
        }
    }
}
