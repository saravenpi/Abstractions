//! The pooled spray flecks and bubbles: thrown by the churn of a source, then left to fly.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::super::eye::Eye;
use super::super::kind::Kind;
use super::super::pool::Pool;
use super::tune::{BUBBLE_RISE, DROP_SIZE, DROPS, SPRAY_DRAG};
use crate::screens::Screen;
use crate::space::Rebased;

/// One pooled spray fleck or bubble, in render space.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Drop {
    /// Velocity in metres per second.
    pub velocity: Vec3,
    /// Unit local up where it was thrown.
    pub up: Vec3,
    /// Height above the sea in metres, followed along `up`.
    pub level: f32,
    /// Gravity in metres per second squared.
    pub gravity: f32,
    /// Seconds since it was thrown.
    pub age: f32,
    /// Seconds it lives for; 0 while parked.
    pub life: f32,
    /// True for a bubble, which rises under the sea; false for spray, which falls back into it.
    pub bubble: bool,
}

/// The pooled drops in the order they are reused.
#[derive(Resource, Debug, Default, Clone)]
pub struct Drops {
    /// Every pooled drop: spray first, bubbles after.
    pub entities: Vec<Entity>,
    /// Next spray fleck and next bubble to reuse, as indices into `entities`.
    pub next: [usize; 2],
    /// True while any drop may still fly.
    pub awake: bool,
}

/// Number of the pooled drops that are bubbles.
pub const BUBBLES: usize = DROPS / 4;

impl Drops {
    /// Returns the next drop of the kind to reuse, the oldest, and moves on past it.
    pub fn take(&mut self, bubble: bool) -> Option<Entity> {
        let (start, count) = match bubble {
            true => (DROPS - BUBBLES, BUBBLES),
            false => (0, DROPS - BUBBLES),
        };
        let slot = &mut self.next[usize::from(bubble)];
        let entity = self.entities.get(start + *slot % count).copied();
        *slot = (*slot + 1) % count;
        self.awake |= entity.is_some();
        entity
    }
}

/// Spawns the parked drops when gameplay starts, unless they already exist.
pub fn spawn(
    mut commands: Commands,
    pool: Res<Pool>,
    mut drops: ResMut<Drops>,
    existing: Query<(), With<Drop>>,
) {
    if !existing.is_empty() {
        return;
    }
    drops.entities.clear();
    for index in 0..DROPS {
        let bubble = index >= DROPS - BUBBLES;
        let kind = [Kind::Spray, Kind::Bubble][usize::from(bubble)];
        let entity = commands.spawn((
            Drop {
                bubble,
                ..Drop::default()
            },
            Mesh3d(pool.disc.clone()),
            MeshMaterial3d(pool.material(kind, 0).clone()),
            Transform::from_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
        drops.entities.push(entity.id());
    }
}

/// Returns `drop` after `dt` seconds: spray flies under gravity and the drag of the air,
/// bubbles rise toward their buoyant speed. Also returns how far it moved.
pub fn fly(drop: &Drop, dt: f32) -> (Drop, Vec3) {
    let velocity = match drop.bubble {
        true => drop
            .velocity
            .lerp(drop.up * BUBBLE_RISE, (4.0 * dt).min(1.0)),
        false => (drop.velocity - drop.up * drop.gravity * dt) * (-SPRAY_DRAG * dt).exp(),
    };
    let moved = velocity * dt;
    let next = Drop {
        velocity,
        level: drop.level + moved.dot(drop.up),
        age: drop.age + dt,
        ..*drop
    };
    (next, moved)
}

/// Returns true while `drop` still flies: within its life, spray above the sea, bubbles under it.
pub fn flying(drop: &Drop) -> bool {
    let wet = drop.level < 0.0;
    drop.age < drop.life && wet == drop.bubble
}

/// Moves every flying drop, facing the eye, and parks the ones that end.
pub fn step(
    time: Res<Time>,
    eye: Res<Eye>,
    mut drops: ResMut<Drops>,
    mut query: Query<(&mut Drop, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let mut awake = false;
    for (mut drop, mut transform) in &mut query {
        if drop.life <= 0.0 {
            continue;
        }
        let (next, moved) = fly(&drop, dt);
        *drop = next;
        if !flying(&drop) {
            drop.life = 0.0;
            transform.scale = Vec3::ZERO;
            continue;
        }
        awake = true;
        let size = [DROP_SIZE.0, DROP_SIZE.1][usize::from(drop.bubble)];
        let fade = ((drop.life - drop.age) * 3.0).clamp(0.0, 1.0);
        transform.translation += moved;
        transform.rotation = eye.rot;
        transform.scale = Vec3::new(size * fade, size * fade, 1.0);
    }
    drops.awake = awake;
}
