//! Flying: a thrown javelin through the world under the gravity and the air of the planet, and its
//! arrival in the ground or an animal. It is let go by [`super::loose`] and taken again by
//! [`super::pull`].

use avian3d::prelude::*;
use bevy::prelude::*;

use super::flight::{Rest, Spear, fly, pierces, vane};
use super::hit::Hit;
use super::javelin::tune::LIE;
use super::javelin::{Ground, Meet, Planet, Quiver, Soil, cross, lie, meet, planted};
use super::landing::{arrival, grounded, steady};
use super::outcome::Outcome;
use super::sense::{Parts, first};
use super::tune::REST_SPEED;
use crate::audio::Impact;
use crate::fauna::Live;
use crate::space::{Anchor, Rebased, local_gravity};
use crate::world::{Chunks, Terrain};

/// Where a step of a javelin meets the world: the point, the unit normal there and what it is.
type Contact = (Vec3, Vec3, Soil);

/// Returns where the step of `spear` from `from` to `to` first meets the colliders of `world` or
/// the drawn `ground`. A collider on the drawn ground is that ground; any other one is hard.
fn met(
    world: &SpatialQuery,
    ground: &impl Ground,
    (spear, from, to): (&Spear, Vec3, Vec3),
) -> Option<Contact> {
    let filter = SpatialQueryFilter::default().with_excluded_entities([spear.thrower]);
    let way = Dir3::new(to - from).ok();
    let hit = way.and_then(|way| {
        let met = world.cast_ray(from, way, from.distance(to), true, &filter)?;
        let point = from + way * met.distance;
        let soil = match ground.clearance(point).abs() < 0.5 {
            true => ground.soil(point),
            false => Soil::Hard,
        };
        Some((point, met.normal, soil))
    });
    let drawn =
        cross(ground, from, to).map(|point| (point, ground.normal(point), ground.soil(point)));
    match (hit, drawn) {
        (Some(a), Some(b)) if b.0.distance(from) < a.0.distance(from) => Some(b),
        (Some(a), _) => Some(a),
        (None, b) => b,
    }
}

/// Lands `spear`, the entity `entity` placed at `place`, on what it met at `contact`: it plants
/// there and quivers, or glances off and, once slow on the ground, lies on it. It sounds a thud on
/// soft ground and a clack where it glances.
fn arrive(
    commands: &mut Commands,
    (entity, spear, place): (Entity, &mut Spear, &mut Transform),
    ground: &impl Ground,
    (point, normal, soil): Contact,
) -> Impact {
    let axis = place.rotation * Vec3::Y;
    let mut impact = grounded(spear, point, soil);
    match meet(spear, axis, (normal, soil)) {
        Meet::Plant(deep) => {
            *place = planted(point, axis, deep);
            commands
                .entity(entity)
                .insert(Quiver::of(place.rotation, axis, spear.velocity));
            (spear.rest, spear.velocity) = (Rest::Stuck, Vec3::ZERO);
        }
        Meet::Glance(velocity) => {
            impact.deflected = true;
            place.translation = point + normal * LIE;
            spear.velocity = velocity;
            if velocity.length() < REST_SPEED && ground.clearance(point).abs() < 0.5 {
                let length = spear.item.build.grip + spear.item.build.reach();
                *place = lie(ground, point, axis, length);
                (spear.rest, spear.velocity) = (Rest::Lying, Vec3::ZERO);
            }
        }
    }
    impact
}

/// What the flight of a javelin reads of the world: where render space sits, the height field and
/// the meshes drawn.
type World<'w> = (Res<'w, Anchor>, Res<'w, Terrain>, Option<Res<'w, Chunks>>);

/// Flies every javelin in the air one frame, under the gravity of the planet and the drag of its
/// air, its shaft turning nose first along its path, and lands it on the first collider or drawn
/// ground its step meets: no frame ever leaves its tip under the ground.
pub fn soar(
    mut commands: Commands,
    time: Res<Time>,
    (anchor, terrain, chunks): World,
    (world, mut sounds): (SpatialQuery, MessageWriter<Impact>),
    mut spears: Query<(Entity, &mut Spear, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let ground = Planet::new(&anchor, &terrain, chunks.as_deref());
    for (entity, mut spear, mut place) in
        spears.iter_mut().filter(|(_, s, _)| s.rest == Rest::Flying)
    {
        let gravity = local_gravity(&anchor, terrain.globe(), place.translation);
        let from = place.translation;
        let (to, velocity) = fly(from, spear.velocity, gravity, spear.item.throw.drag, dt);
        spear.was = from;
        if let Some(contact) = met(&world, &ground, (&*spear, from, to)) {
            sounds.write(arrive(
                &mut commands,
                (entity, &mut *spear, &mut *place),
                &ground,
                contact,
            ));
            continue;
        }
        let axis = vane(
            place.rotation * Vec3::Y,
            velocity,
            spear.item.throw.stable,
            dt,
        );
        place.translation = to;
        place.rotation = Quat::from_rotation_arc(Vec3::Y, axis);
        spear.velocity = velocity;
    }
}

/// Lands every javelin in the air on the first animal part its last step went through: it stays in
/// the animal when the blow gets through, else it falls away.
pub fn pierce(
    mut commands: Commands,
    mut spears: Query<(Entity, &mut Spear, &GlobalTransform)>,
    (parts, creatures, roots): (Parts, Query<(), Live>, Query<&GlobalTransform, Live>),
    mut hits: MessageWriter<Hit>,
) {
    for (entity, mut spear, placed) in &mut spears {
        let now = placed.translation();
        if !steady(&spear, now) {
            continue;
        }
        let pad = spear.item.build.head.width() / 2.0;
        let Some((part, victim, at)) = first(&parts, &creatures, ([spear.was; 2], [now; 2]), pad)
        else {
            continue;
        };
        let mut hit = arrival(&spear, (victim, part, at));
        let stuck = pierces(&hit.blow);
        if !stuck {
            hit.outcome = Outcome::Deflected;
        }
        hits.write(hit);
        match (stuck, roots.get(victim)) {
            (true, Ok(root)) => {
                spear.rest = Rest::Stuck;
                commands
                    .entity(entity)
                    .remove::<Rebased>()
                    .insert((placed.reparented_to(root), ChildOf(victim)));
            }
            _ => spear.velocity = -spear.velocity * super::tune::RESTITUTION,
        }
    }
}
