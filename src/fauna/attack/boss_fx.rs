//! Visual telegraphs and arcing projectiles for boss special attacks.

use bevy::prelude::*;

use crate::audio::{Body, Impact, Outer};
use crate::avatar::blow::Blow;
use crate::life::Reaction;
use crate::player::Player;
use crate::screens::Screen;

/// Expanding crimson warning ring spawned on the ground during a boss [`super::boss::BossSpecial::SeismicSlam`].
#[derive(Component, Debug, Clone, Copy)]
pub struct SlamRing {
    /// Seconds elapsed since the ring spawned.
    pub age: f32,
    /// Total windup duration in seconds before despawning.
    pub duration: f32,
    /// Target outer radius in metres at full expansion.
    pub max_radius: f32,
}

/// Arcing bioluminescent projectile fired by a boss [`super::boss::BossSpecial::BioBarrage`].
#[derive(Component, Debug, Clone, Copy)]
pub struct BossProjectile {
    /// Current velocity in render space, metres per second.
    pub vel: Vec3,
    /// Seconds elapsed since launch.
    pub age: f32,
    /// Impact energy in joules.
    pub joules: f32,
    /// Impact momentum in kg·m/s.
    pub momentum: f32,
}

/// Returns the emissive tint for a boss projectile from the species' tissue `reaction`.
fn reaction_colour(reaction: Reaction) -> (Color, LinearRgba) {
    match reaction {
        Reaction::Acid => (
            Color::srgb(0.3, 0.95, 0.2),
            LinearRgba::new(0.6, 5.0, 0.3, 1.0),
        ),
        Reaction::Venom => (
            Color::srgb(0.75, 0.2, 0.95),
            LinearRgba::new(3.5, 0.4, 5.0, 1.0),
        ),
        Reaction::Charge => (
            Color::srgb(0.2, 0.85, 1.0),
            LinearRgba::new(0.4, 3.8, 5.5, 1.0),
        ),
        Reaction::Inert => (
            Color::srgb(1.0, 0.25, 0.12),
            LinearRgba::new(5.5, 0.9, 0.3, 1.0),
        ),
    }
}

/// Spawns an expanding glowing crimson ground warning ring at `pos` oriented with `up`.
pub(super) fn spawn_ring(
    commands: &mut Commands,
    (meshes, materials): (&mut Assets<Mesh>, &mut Assets<StandardMaterial>),
    (pos, up): (Vec3, Vec3),
    (duration, max_radius): (f32, f32),
) {
    let mesh = meshes.add(Torus::new(0.9, 1.0));
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.12, 0.1, 0.85),
        emissive: LinearRgba::new(6.0, 0.4, 0.2, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let rot = Quat::from_rotation_arc(Vec3::Y, up.normalize_or(Vec3::Y));
    commands.spawn((
        SlamRing {
            age: 0.0,
            duration,
            max_radius,
        },
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_translation(pos + up * 0.12)
            .with_rotation(rot)
            .with_scale(Vec3::new(0.3, 0.15, 0.3)),
        DespawnOnEnter(Screen::Title),
    ));
}

/// Spawns `count` glowing arcing projectiles fanned around `dir` from `origin`.
pub(super) fn spawn_barrage(
    commands: &mut Commands,
    (meshes, materials): (&mut Assets<Mesh>, &mut Assets<StandardMaterial>),
    (origin, dir, up): (Vec3, Vec3, Vec3),
    (count, reaction, mass): (usize, Reaction, f32),
) {
    let (base, emissive) = reaction_colour(reaction);
    let mesh = meshes.add(Sphere::new(0.32));
    let mat = materials.add(StandardMaterial {
        base_color: base,
        emissive,
        unlit: true,
        ..default()
    });
    let joules = (120.0 + 0.008 * mass).clamp(120.0, 480.0);
    let momentum = (60.0 + 0.004 * mass).clamp(60.0, 240.0);
    let mid = (count as f32 - 1.0) * 0.5;
    for i in 0..count {
        let angle = (i as f32 - mid) * 0.16;
        let aim = Quat::from_axis_angle(up, angle) * dir.normalize_or(Vec3::NEG_Z);
        let vel = aim * 24.0 + up * 7.5;
        commands.spawn((
            BossProjectile {
                vel,
                age: 0.0,
                joules,
                momentum,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(origin),
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Expands active [`SlamRing`] warning telegraphs and despawns them once their windup ends.
pub fn tick_rings(
    mut commands: Commands,
    time: Res<Time>,
    mut rings: Query<(Entity, &mut SlamRing, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.1);
    for (entity, mut ring, mut tf) in &mut rings {
        ring.age += dt;
        if ring.age >= ring.duration {
            commands.entity(entity).despawn();
            continue;
        }
        let t = (ring.age / ring.duration.max(1e-3)).clamp(0.0, 1.0);
        let r = (0.3 + (ring.max_radius - 0.3) * t).max(0.3);
        tf.scale = Vec3::new(r, 0.15, r);
    }
}

/// Emits a heavy impact sound at `at` with `joules`.
pub(super) fn rumble(sounds: &mut MessageWriter<Impact>, at: Vec3, joules: f32) {
    sounds.write(Impact {
        at,
        joules,
        cut: false,
        outer: Outer::RobotPlate,
        struck: Body {
            stiffness: 180.0,
            mass: 80.0,
        },
        tool: Body {
            stiffness: 40.0,
            mass: 400.0,
        },
        deflected: false,
    });
}

/// Advances arcing [`BossProjectile`] meshes, checks impact against the local player, and despawns
/// expired bolts.
pub fn tick_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    robot: Query<&Transform, (With<Player>, Without<BossProjectile>)>,
    mut projs: Query<(Entity, &mut BossProjectile, &mut Transform)>,
    (mut blows, mut sounds): (MessageWriter<Blow>, MessageWriter<Impact>),
) {
    let dt = time.delta_secs().min(0.1);
    let player_pos = robot.single().ok().map(|t| t.translation);
    for (entity, mut proj, mut tf) in &mut projs {
        proj.age += dt;
        proj.vel.y -= 12.0 * dt;
        tf.translation += proj.vel * dt;
        let hit = player_pos.is_some_and(|p| p.distance(tf.translation) <= 1.35);
        if hit {
            blows.write(Blow {
                at: tf.translation,
                joules: proj.joules,
                cut: false,
                dir: proj.vel.normalize_or(Vec3::NEG_Z),
                momentum: proj.momentum,
            });
            rumble(&mut sounds, tf.translation, proj.joules);
        }
        if hit || proj.age >= 3.2 {
            commands.entity(entity).despawn();
        }
    }
}
