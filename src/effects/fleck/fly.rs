//! Flying flecks: thrown into a free slot, then moved under gravity and drag until they land.

use bevy::prelude::*;

use super::pool::{Fleck, Flecks, claim};
use super::tune::LIFE;
use super::{Fling, Landed};
use crate::life::blood::drag;
use crate::space::{Anchor, altitude, local_gravity};
use crate::wind::Thickness;
use crate::world::Terrain;

/// Returns where a fleck of `radius` metres goes from `at` with `velocity` in `dt` seconds under
/// `gravity` in air `thick` times as dense as at home, and its new velocity: the speed falls as
/// e^(-k·distance) (the quadratic drag of a sphere), then the step moves it (semi-implicit Euler).
pub fn step(
    at: Vec3,
    velocity: Vec3,
    (gravity, thick): (Vec3, f32),
    (radius, dt): (f32, f32),
) -> (Vec3, Vec3) {
    let velocity = velocity + gravity * dt;
    let velocity = velocity * (-drag(radius, thick) * velocity.length() * dt).exp();
    (at + velocity * dt, velocity)
}

/// Takes a slot for every fleck thrown this frame.
pub fn fling(
    mut flings: MessageReader<Fling>,
    mut flecks: ResMut<Flecks>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut slots: Query<(
        &mut Fleck,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for f in flings.read() {
        let slot = claim(slots.iter().map(|(s, ..)| s.live.then_some(s.age)));
        let Some((mut fleck, mut place, mut shown, mut look)) = slots.iter_mut().nth(slot) else {
            continue;
        };
        if !fleck.live {
            flecks.live += 1;
        }
        *fleck = Fleck {
            live: true,
            velocity: f.velocity,
            age: 0.0,
            radius: f.radius,
            tag: f.tag,
        };
        *place = Transform::from_translation(f.at).with_scale(Vec3::splat(2.0 * f.radius));
        *shown = Visibility::Inherited;
        look.0 = flecks.look(&mut materials, f.colour, f.wet);
    }
}

/// Returns the height in metres above sea level of the ground or the liquid under render position
/// `pos`: what a fleck lands on.
fn floor(anchor: &Anchor, terrain: &Terrain, pos: Vec3) -> f32 {
    let at = terrain.locate(anchor.to_frame(pos));
    terrain.height(at.x, at.z).max(terrain.globe().sea_level())
}

/// Flies every live fleck one frame, and lets go of the ones that landed or lived too long. A fleck
/// that lands on dry ground sends a [`Landed`].
pub fn fly(
    time: Res<Time>,
    (anchor, terrain, air): (Res<Anchor>, Res<Terrain>, Res<Thickness>),
    mut flecks: ResMut<Flecks>,
    mut slots: Query<(&mut Fleck, &mut Transform, &mut Visibility)>,
    mut landed: MessageWriter<Landed>,
) {
    let dt = time.delta_secs();
    for (mut fleck, mut place, mut shown) in &mut slots {
        if !fleck.live {
            continue;
        }
        let pull = local_gravity(&anchor, terrain.globe(), place.translation);
        fleck.age += dt;
        let at = place.translation;
        let (to, velocity) = step(at, fleck.velocity, (pull, air.0), (fleck.radius, dt));
        let ground = floor(&anchor, &terrain, to);
        let low = altitude(&anchor, terrain.globe(), to) <= ground + fleck.radius;
        if low || fleck.age > LIFE {
            let dry = ground > terrain.globe().sea_level();
            if low && dry {
                landed.write(Landed {
                    at: to,
                    radius: fleck.radius,
                    tag: fleck.tag,
                    heading: velocity.normalize_or_zero(),
                });
            }
            fleck.live = false;
            flecks.live = flecks.live.saturating_sub(1);
            *shown = Visibility::Hidden;
            continue;
        }
        fleck.velocity = velocity;
        place.translation = to;
    }
}
