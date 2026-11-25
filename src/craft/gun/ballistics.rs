//! Ballistic tracer spawning, gravity integration, and world, fauna, and PvP hit resolution.

use abstractions_net::wire::Blow as Told;
use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::{BulletTracer, GunKind, ShotFired};
use crate::avatar::Own;
use crate::avatar::strike::sense::{Parts, first};
use crate::avatar::strike::{Hit, Land, Meets, Outcome, Solids, Tip};
use crate::craft::build::{Blow, Intent, Mode};
use crate::craft::matter::{Mineral, bullet_mass};
use crate::craft::pack::tune::STONE_TONES;
use crate::fauna::Live;
use crate::net::{Link, Remote, Remotes, hooks};
use crate::screens::Screen;
use crate::space::{Anchor, up};

/// Radius around a remote robot centre matched by bullet segments, in metres.
const PUPPET_RADIUS: f32 = 0.7;

/// World collision, fauna hitboxes, and local player identity read during tracer stepping.
#[derive(SystemParam)]
pub struct TraceWorld<'w, 's> {
    /// Avian spatial query for solid world colliders.
    world: SpatialQuery<'w, 's>,
    /// Planet anchor, terrain, and chunk data for drawn ground collision.
    land: Land<'w>,
    /// Fauna part bounding boxes.
    parts: Parts<'w, 's>,
    /// Living creature marker query.
    creatures: Query<'w, 's, (), Live>,
    /// Local player and body entity lookup.
    own: Own<'w, 's>,
}

/// Hit outputs and multiplayer state used while stepping tracers.
#[derive(SystemParam)]
pub struct TraceHits<'w, 's> {
    /// Fauna hit message writer.
    hits: MessageWriter<'w, Hit>,
    /// Remote players tracked in a networked session.
    remotes: Option<Res<'w, Remotes>>,
    /// Network link used to send PvP hits.
    link: Option<ResMut<'w, Link>>,
    /// Remote robot puppet transforms.
    puppets: Query<'w, 's, &'static GlobalTransform, With<Remote>>,
}

/// Returns impact energy in joules scaled from chassis `damage` and bullet `ammo` density.
pub fn bullet_energy(damage: f32, ammo: Mineral) -> f32 {
    let density_factor = (ammo.material().density / 2600.0).sqrt().clamp(0.7, 1.5);
    2.2 * damage * density_factor
}

/// Builds and spawns a single tracer entity for `shot`.
fn spawn_one(
    commands: &mut Commands,
    (mesh, mat): (Handle<Mesh>, Handle<StandardMaterial>),
    shot: &ShotFired,
    shooter: Option<Entity>,
) {
    let dir = shot.dir.normalize_or(Vec3::NEG_Z);
    let stats = shot.kind.stats();
    let tracer = BulletTracer {
        prev: shot.origin,
        pos: shot.origin,
        vel: dir * stats.speed,
        life: 2.0,
        damage: stats.damage,
        ammo: shot.ammo,
        shooter: shot.local.then_some(shooter).flatten(),
        rifle: shot.kind == GunKind::MineralRifle,
        local: shot.local,
    };
    let at = Transform::from_translation(shot.origin)
        .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir));
    let look = (Mesh3d(mesh), MeshMaterial3d(mat), NotShadowCaster);
    commands.spawn((tracer, look, at, DespawnOnExit(Screen::Gameplay)));
}

/// Spawns glowing [`BulletTracer`] entities for every [`ShotFired`] message.
pub fn spawn_tracers(
    mut commands: Commands,
    mut shots: MessageReader<ShotFired>,
    own: Own,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    mut bolt: Local<Option<Handle<Mesh>>>,
) {
    let shooter = own.body().map(|(player, _)| player);
    for shot in shots.read() {
        let mesh = bolt
            .get_or_insert_with(|| meshes.add(Cuboid::new(0.024, 0.024, 0.36)))
            .clone();
        let rgb = STONE_TONES[shot.ammo as usize];
        let emissive = LinearRgba::rgb(rgb[0] * 4.5 + 1.4, rgb[1] * 4.5 + 1.0, rgb[2] * 4.5 + 0.6);
        let mat = materials.add(StandardMaterial {
            base_color: Color::linear_rgb(rgb[0], rgb[1], rgb[2]),
            emissive,
            ..default()
        });
        spawn_one(&mut commands, (mesh, mat), shot, shooter);
    }
}

/// Returns the closest point along segment `seg` within [`PUPPET_RADIUS`] of `centre`.
fn puppet_reach(seg: [Vec3; 2], centre: Vec3) -> Option<Vec3> {
    let span = seg[1] - seg[0];
    let share = (centre - seg[0]).dot(span) / span.length_squared().max(f32::EPSILON);
    let nearest = seg[0] + span * share.clamp(0.0, 1.0);
    (nearest.distance(centre) < PUPPET_RADIUS).then_some(nearest)
}

/// Builds a [`Hit`] message for a bullet striking `victim`'s `part` at `at`.
fn make_hit(
    attacker: Entity,
    (victim, part, at): (Entity, Entity, Vec3),
    (along, damage, ammo): (Vec3, f32, Mineral),
) -> Hit {
    let mat = ammo.material();
    let energy = bullet_energy(damage, ammo);
    let blow = Blow {
        energy,
        mode: Mode::Stab,
        width: 0.012,
        area: 1.2e-4,
        hardness: mat.hardness,
    };
    let tip = Tip {
        stiffness: mat.stiffness,
        mass: 0.04,
    };
    Hit {
        attacker,
        item: u32::MAX,
        victim,
        part,
        at,
        along,
        blow,
        intent: Intent::Pierce,
        tip,
        outcome: Outcome::Clean,
    }
}

/// Checks segment `seg` against live fauna parts and emits a [`Hit`] when struck.
fn fauna_hit(
    tracer: &BulletTracer,
    seg: [Vec3; 2],
    world: &TraceWorld,
    hits: &mut MessageWriter<Hit>,
) -> bool {
    let Some((part, victim, at)) = first(&world.parts, &world.creatures, (seg, seg), 0.14) else {
        return false;
    };
    let attacker = world.own.body().map_or(victim, |(_, body)| body);
    let along = tracer.vel.normalize_or(Vec3::NEG_Z);
    hits.write(make_hit(
        attacker,
        (victim, part, at),
        (along, tracer.damage, tracer.ammo),
    ));
    true
}

/// Checks segment `seg` against remote player puppets and sends a network blow if struck.
fn pvp_hit(
    tracer: &BulletTracer,
    seg: [Vec3; 2],
    anchor: Option<&Anchor>,
    hits: &mut TraceHits,
) -> bool {
    let (Some(anchor), Some(remotes), Some(link)) =
        (anchor, hits.remotes.as_deref(), hits.link.as_deref_mut())
    else {
        return false;
    };
    let Some(told) = remotes.robots.iter().find_map(|(victim, track)| {
        let body = hits.puppets.get(track.puppet?).ok()?;
        let at = puppet_reach(seg, body.translation())?;
        let normal = up(anchor, body.translation())
            .as_dvec3()
            .normalize_or(DVec3::Y);
        let turn = DQuat::from_rotation_arc(DVec3::Y, normal)
            .as_quat()
            .inverse();
        let dir = tracer.vel.normalize_or(Vec3::NEG_Z);
        Some(Told {
            victim: *victim,
            at: (turn * (at - body.translation())).to_array(),
            joules: bullet_energy(tracer.damage, tracer.ammo),
            cut: true,
            dir: (turn * dir).to_array(),
            momentum: bullet_mass(tracer.ammo) * tracer.vel.length(),
        })
    }) else {
        return false;
    };
    hooks::send_blow(link, told);
    true
}

/// Integrates gravity on flying tracers and resolves solid, fauna, and remote PvP impacts.
pub fn step_tracers(
    mut commands: Commands,
    time: Res<Time>,
    world: TraceWorld,
    mut hits: TraceHits,
    mut tracers: Query<(Entity, &mut BulletTracer, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tracer, mut place) in &mut tracers {
        let up_dir = world
            .land
            .0
            .as_deref()
            .map_or(Vec3::Y, |a| up(a, tracer.pos));
        let prev = tracer.pos;
        tracer.vel += -4.5 * up_dir * dt;
        let next = prev + tracer.vel * dt;
        (tracer.prev, tracer.pos) = (prev, next);
        tracer.life -= dt;
        place.translation = next;
        place.rotation = Quat::from_rotation_arc(Vec3::Z, tracer.vel.normalize_or(Vec3::NEG_Z));
        let own = tracer.shooter.unwrap_or(Entity::PLACEHOLDER);
        let solid = Solids::new(&world.world, &world.land, own).meet(prev, next);
        let seg = [prev, solid.map_or(next, |touch| touch.at)];
        let hit = tracer.local
            && (fauna_hit(&tracer, seg, &world, &mut hits.hits)
                || pvp_hit(&tracer, seg, world.land.0.as_deref(), &mut hits));
        if hit || solid.is_some() || tracer.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
