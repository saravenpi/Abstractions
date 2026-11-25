//! Creature collisions against static solids (base, rocks, trees, ships), each other and
//! standing robots. Runs after `walk` and `wing::clear` inside `Posed`, before `bulk_up`.

pub(in crate::fauna) mod math;
mod solid;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::math::{DVec3, Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::*;

use self::math::{
    Actor, Parts, SOLID_NEAR, apply_shift, capsule_span, push_from_robot, separate_pair,
    strip_inward,
};
use super::beast::{Beast, Part};
use super::sync::RemoteFauna;
use super::tune::MAX_STEP;
use super::want::Lead;
use super::{CaveBound, Dead, Frozen, near, walk, wing};
use crate::avatar::Avatar;
use crate::life::Species;
use crate::life::body::Locomotion;
use crate::player::Player;
use crate::screens::playing;
use crate::space::{Anchor, Frame, Location, Travel};
use crate::world::Terrain;

/// Query selecting the local standing player transform without child parts.
type PlayerPose<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), (With<Player>, Without<Part>)>;

/// Query filter selecting standing remote robot avatars without child parts.
type RemoteFilter = (
    With<Avatar>,
    Without<ChildOf>,
    Without<Player>,
    Without<Part>,
);

/// Query selecting standing remote robot transforms.
type RemotePoses<'w, 's> = Query<'w, 's, &'static Transform, RemoteFilter>;

/// Read-only world context needed to resolve creature collisions.
#[derive(SystemParam)]
pub struct Surround<'w, 's> {
    /// Terrain height field of the current planet.
    pub(super) terrain: Res<'w, Terrain>,
    /// Current render anchor.
    pub(super) anchor: Res<'w, Anchor>,
    /// Frame clock.
    pub(super) time: Res<'w, Time>,
    /// Player planet and travel state.
    pub(super) location: Res<'w, Location>,
    /// Local player entity and transform.
    pub(super) player: PlayerPose<'w, 's>,
    /// Remote standing robot transforms.
    pub(super) remotes: RemotePoses<'w, 's>,
    /// World collider shapes used to filter out terrain trimeshes.
    pub(super) colliders: Query<'w, 's, &'static Collider, (Without<Beast>, Without<Part>)>,
    /// Spatial query access over world colliders.
    pub(super) query: SpatialQuery<'w, 's>,
    /// Remotely steered creatures exempt from local collision pushes.
    pub(super) remote: Option<Res<'w, RemoteFauna>>,
}

/// Scratch buffers reused across frames so collision resolution never allocates in steady state.
#[derive(Default)]
pub struct Scratch {
    /// Living drawn beasts collected this frame.
    actors: Vec<Actor>,
    /// Planet positions of all standing local and remote robots.
    robots: Vec<DVec3>,
}

/// Living beasts that participate in collision response.
type Living = (Without<Dead>, Without<Frozen>);

/// Query over living beasts with their optional cave bounds.
type Beasts<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Beast, Option<&'static CaveBound>), Living>;

/// Returns true for open-air flyers whose world avoidance runs in `wing::clear`.
fn aloft(locomotion: Locomotion) -> bool {
    matches!(
        locomotion,
        Locomotion::Fly | Locomotion::Swarm | Locomotion::Pulse
    )
}

/// Fills `scratch` with living drawn beasts and standing robots for this frame.
fn gather(surround: &Surround, beasts: &Beasts, species: &[Species], scratch: &mut Scratch) {
    scratch.actors.clear();
    scratch.robots.clear();
    if surround.location.travel == Travel::OnFoot
        && let Ok((_, t)) = surround.player.single()
    {
        scratch.robots.push(surround.anchor.to_frame(t.translation));
    }
    for t in &surround.remotes {
        scratch.robots.push(surround.anchor.to_frame(t.translation));
    }
    for (entity, beast, _) in beasts {
        if matches!(beast.lead, Lead::Carcass { .. })
            || surround
                .remote
                .as_deref()
                .is_some_and(|r| r.contains(beast.lead.key()))
        {
            continue;
        }
        let Some(spec) = species.get(usize::from(beast.index)) else {
            continue;
        };
        let pos = beast.spot.planet(beast.motion.root.at);
        let (rad, r_min, r_max) = capsule_span(pos, &spec.body);
        scratch.actors.push(Actor {
            entity,
            species: usize::from(beast.index),
            shift: Vec2::ZERO,
            pos,
            rad,
            r_min,
            r_max,
            mass: spec.history.mass.max(1.0),
            spot: beast.spot,
        });
    }
}

/// Accumulates mass-weighted separation between overlapping creatures and removes closing speed.
fn separate_beasts(scratch: &mut Scratch, beasts: &mut Beasts) {
    let count = scratch.actors.len();
    for i in 0..count {
        for j in (i + 1)..count {
            let Some((pa, pb)) = separate_pair(&scratch.actors[i], &scratch.actors[j]) else {
                continue;
            };
            scratch.actors[i].shift += pa;
            scratch.actors[j].shift += pb;
            if let Ok((_, mut ba, _)) = beasts.get_mut(scratch.actors[i].entity) {
                strip_inward(&mut ba.motion.root.vel.at, pa);
            }
            if let Ok((_, mut bb, _)) = beasts.get_mut(scratch.actors[j].entity) {
                strip_inward(&mut bb.motion.root.vel.at, pb);
            }
        }
    }
}

/// Accumulates creature separation against all standing local and remote robots.
fn separate_robots(scratch: &mut Scratch, species: &[Species], beasts: &mut Beasts) {
    for actor in &mut scratch.actors {
        let body = &species[actor.species].body;
        let Ok((_, mut beast, _)) = beasts.get_mut(actor.entity) else {
            continue;
        };
        let facing = (beast.motion.root.turn * Vec3::NEG_Z).xz();
        for &robot in &scratch.robots {
            let at = beast.motion.root.at + Vec3::new(actor.shift.x, 0.0, actor.shift.y);
            if let Some(push) = push_from_robot(&beast.spot, (at, facing), body, robot) {
                actor.shift += push;
                strip_inward(&mut beast.motion.root.vel.at, push);
            }
        }
    }
}

/// Sweeps near walkers against static solids and syncs shifted beasts, feet and child parts.
fn finish_beasts(
    surround: &Surround,
    species: &[Species],
    scratch: &Scratch,
    beasts: &mut Beasts,
    parts: &mut Parts,
) {
    let player = surround.player.single().ok();
    let player_entity = player.map(|(e, _)| e);
    let player_pos = player.map(|(_, t)| surround.anchor.to_frame(t.translation));
    let on_surface = surround.anchor.frame == Frame::Surface(surround.location.body);
    let dt = surround.time.delta_secs().min(MAX_STEP);
    for actor in &scratch.actors {
        let Ok((_, mut beast, cave)) = beasts.get_mut(actor.entity) else {
            continue;
        };
        let body = &species[actor.species].body;
        let before_xz = beast.motion.root.at.xz();
        let desired_xz = before_xz + actor.shift;
        let limit = SOLID_NEAR + f64::from(body.length);
        let near = player_pos
            .is_some_and(|p| beast.spot.planet(beast.motion.root.at).distance(p) <= limit);
        let final_xz = if on_surface && near && !aloft(body.locomotion) {
            solid::sweep(surround, &mut beast, body, desired_xz, (player_entity, dt))
        } else {
            desired_xz
        };
        apply_shift(
            (&mut beast, cave),
            body,
            &surround.terrain,
            final_xz - before_xz,
            parts,
        );
    }
}

/// Resolves creature collisions against each other, robots and nearby static world solids.
pub fn resolve(
    surround: Surround,
    mut beasts: Beasts,
    mut parts: Parts,
    mut scratch: Local<Scratch>,
) {
    let Some(life) = surround.terrain.globe().life() else {
        return;
    };
    let species = &life.species;
    gather(&surround, &beasts, species, &mut scratch);
    separate_beasts(&mut scratch, &mut beasts);
    separate_robots(&mut scratch, species, &mut beasts);
    finish_beasts(&surround, species, &scratch, &mut beasts, &mut parts);
}

/// Registers creature collision resolution after movement and flight avoidance.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        resolve
            .in_set(super::Posed)
            .after(walk::walk)
            .after(wing::clear)
            .before(near::mark)
            .before(super::bulk::bulk_up)
            .run_if(playing)
            .run_if(super::gate::living)
            .run_if(resource_exists::<avian3d::collider_tree::ColliderTrees>)
            .run_if(any_with_component::<Beast>),
    );
}
