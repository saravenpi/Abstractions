//! Flying: a thrown javelin through the world under the gravity and the air of the planet, and its
//! arrival in the ground or an animal. It is let go by [`super::loose`] and taken again by
//! [`super::pull`].

use avian3d::prelude::*;
use bevy::prelude::*;

use super::flight::{Rest, Spear, fly, pierces, vane};
use super::hit::Hit;
use super::javelin::tune::LIE;
use super::javelin::{Ground, Meet, Planet, Quiver, lie, meet, planted};
use super::landing::{arrival, grounded, steady};
use super::outcome::Outcome;
use super::sense::{Parts, first};
use super::solid::{Land, Meets, Solids, Touch};
use super::tune::REST_SPEED;
use crate::audio::Impact;
use crate::fauna::Live;
use crate::space::{Rebased, local_gravity};

/// Lands `spear`, the entity `entity` placed at `place`, on what it met at `touch`: it plants
/// there and quivers, or glances off and, once slow on the ground, lies on it. It sounds a thud on
/// soft ground and a clack where it glances.
fn arrive(
    commands: &mut Commands,
    (entity, spear, place): (Entity, &mut Spear, &mut Transform),
    ground: &impl Ground,
    touch: Touch,
) -> Impact {
    let Touch {
        at: point,
        normal,
        soil,
        ..
    } = touch;
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

/// Flies every javelin in the air one frame, under the gravity of the planet and the drag of its
/// air, its shaft turning nose first along its path, and lands it on the first collider or drawn
/// ground its step meets: no frame ever leaves its tip under the ground.
pub fn soar(
    mut commands: Commands,
    time: Res<Time>,
    land: Land,
    (world, mut sounds): (SpatialQuery, MessageWriter<Impact>),
    mut spears: Query<(Entity, &mut Spear, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let (Some(anchor), Some(terrain)) = (land.0.as_deref(), land.1.as_deref()) else {
        return;
    };
    let ground = Planet::new(anchor, terrain, land.2.as_deref());
    for (entity, mut spear, mut place) in
        spears.iter_mut().filter(|(_, s, _)| s.rest == Rest::Flying)
    {
        let gravity = local_gravity(anchor, terrain.globe(), place.translation);
        let from = place.translation;
        let (to, velocity) = fly(from, spear.velocity, gravity, spear.item.throw.drag, dt);
        spear.was = from;
        if let Some(touch) = Solids::new(&world, &land, spear.thrower).meet(from, to) {
            let into = (entity, &mut *spear, &mut *place);
            sounds.write(arrive(&mut commands, into, &ground, touch));
            continue;
        }
        let (stable, axis) = (spear.item.throw.stable, place.rotation * Vec3::Y);
        place.translation = to;
        place.rotation = Quat::from_rotation_arc(Vec3::Y, vane(axis, velocity, stable, dt));
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
