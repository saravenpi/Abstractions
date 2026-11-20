//! Throwing: a drawn spear let go along the view, its flight through the world, its arrival in the
//! ground or an animal, and picking it up again.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::super::Avatar;
use super::act::Strike;
use super::flight::{Rest, Spear, bounce, depth, fly, pierces, stands};
use super::held::Held;
use super::hit::Hit;
use super::hurt::steadied;
use super::input::{Loose, Robot};
use super::pull::{Reach, take};
use super::sense::{Parts, first};
use super::show::{Kit, tool};
use super::tune::{LOFT, PICK, REST_SPEED};
use crate::craft::Pack;
use crate::craft::build::{Intent, blow};
use crate::fauna::Live;
use crate::player::Player;
use crate::screens::Screen;
use crate::space::{Anchor, Rebased, local_gravity};
use crate::view::Heading;
use crate::world::Terrain;

/// Returns `held` flying from `point` along `aim` at its launch speed times the length of `aim`,
/// thrown by the player `by.0` from the body `by.1`.
fn launched(held: &Held, aim: Vec3, by: (Entity, Entity), point: Vec3) -> Spear {
    Spear {
        item: Held {
            out: true,
            ..held.clone()
        },
        velocity: aim * held.throw.launch,
        rest: Rest::Flying,
        thrower: by.0,
        body: by.1,
        was: point,
        lent: None,
    }
}

/// Returns the hit `spear` lands now on `part` of `victim` at `at`, carrying its kinetic energy
/// ½mv².
fn arrival(spear: &Spear, (victim, part, at): (Entity, Entity, Vec3)) -> Hit {
    let speed = spear.velocity.length();
    let mut landed = blow(&spear.item.build, Intent::Throw, &spear.item.place);
    landed.energy = 0.5 * spear.item.throw.mass * speed * speed;
    Hit {
        attacker: spear.body,
        item: spear.item.id,
        victim,
        part,
        at,
        along: spear.velocity / speed.max(f32::EPSILON),
        blow: landed,
        intent: Intent::Throw,
    }
}

/// Returns true when `spear` is in the air and its point moved to `now` by no more than a frame of
/// flight could take it: a rebase of the world in between is not a path through it.
fn steady(spear: &Spear, now: Vec3) -> bool {
    spear.rest == Rest::Flying && now.distance(spear.was) <= spear.velocity.length() * 0.2 + 1.0
}

/// What a throw reads: the players, where they look and the kit that draws the spear.
type Aim<'w, 's> = (
    Query<'w, 's, &'static Transform, With<Player>>,
    Res<'w, Heading>,
    Option<Res<'w, Kit>>,
    Res<'w, Time>,
);

/// Throws the spear of every body that let one go: from the grip, along the view tilted up a
/// little, at the launch speed of the build times the square root of the draw.
pub fn loose(
    mut commands: Commands,
    mut loosed: MessageReader<Loose>,
    bodies: Query<(&Held, &Strike, &ChildOf), With<Avatar>>,
    (players, heading, kit, time): Aim,
    mut pack: Option<ResMut<Pack>>,
) {
    let Some(kit) = kit else { return };
    for loose in loosed.read() {
        let Ok((held, strike, of)) = bodies.get(loose.body) else {
            continue;
        };
        let Ok(player) = players.get(of.parent()) else {
            continue;
        };
        let look = Quat::from_rotation_y(heading.yaw) * Quat::from_rotation_x(heading.pitch + LOFT);
        let aim = player.rotation * look * Vec3::NEG_Z;
        let key = (time.elapsed_secs() * 1000.0) as u32;
        let aim = steadied(aim, strike.hurt.steady, key);
        let point = player.transform_point(strike.blade[0][1]);
        let mut spear = launched(
            held,
            aim * loose.share.sqrt() * strike.hurt.swing,
            (of.parent(), loose.body),
            point,
        );
        spear.lent = pack.as_mut().and_then(|pack| pack.lend(held.id));
        let reach = held.build.reach();
        let drawn = tool(&mut commands, &kit, &held.build, None, -reach);
        let place =
            Transform::from_translation(point).with_rotation(Quat::from_rotation_arc(Vec3::Y, aim));
        commands
            .entity(drawn)
            .insert((spear, place, Rebased, DespawnOnEnter(Screen::Title)));
        commands.entity(loose.body).remove::<Held>();
    }
}

/// Returns the spear `spear` at `place` after it met the ground at `point` with unit `normal`: it
/// stands in it when it arrives point first with the work to drive in, else it bounces, and comes
/// to rest once slow.
fn arrive(spear: &mut Spear, place: &mut Transform, point: Vec3, normal: Vec3) {
    let speed = spear.velocity.length();
    let energy = 0.5 * spear.item.throw.mass * speed * speed;
    let area = spear.item.build.head.area();
    if stands(spear.item.throw.stable, energy, area) {
        let way = spear.velocity / speed.max(f32::EPSILON);
        place.translation = point + way * depth(energy, area).min(spear.item.build.head.size());
        spear.rest = Rest::Stuck;
        return;
    }
    spear.velocity = bounce(spear.velocity, normal);
    place.translation = point + normal * 0.02;
    if spear.velocity.length() < REST_SPEED {
        spear.rest = Rest::Lying;
    }
}

/// Flies every spear in the air one frame, under the gravity of the planet and the drag of its
/// air, and lands it on whatever the world has in its way.
pub fn soar(
    time: Res<Time>,
    (anchor, terrain): (Res<Anchor>, Res<Terrain>),
    world: SpatialQuery,
    mut spears: Query<(&mut Spear, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut spear, mut place) in &mut spears {
        if spear.rest != Rest::Flying {
            continue;
        }
        let gravity = local_gravity(&anchor, terrain.globe(), place.translation);
        let from = place.translation;
        let (to, velocity) = fly(from, spear.velocity, gravity, spear.item.throw.drag, dt);
        spear.was = from;
        let filter = SpatialQueryFilter::default().with_excluded_entities([spear.thrower]);
        let way = Dir3::new(to - from).ok();
        let met = way.and_then(|way| world.cast_ray(from, way, from.distance(to), true, &filter));
        if let (Some(way), Some(met)) = (way, met) {
            arrive(
                &mut spear,
                &mut place,
                from + way * met.distance,
                met.normal,
            );
            continue;
        }
        (place.translation, spear.velocity) = (to, velocity);
        if spear.item.throw.stable && velocity.length() > REST_SPEED {
            place.rotation = Quat::from_rotation_arc(Vec3::Y, velocity.normalize());
        }
    }
}

/// Lands every spear in the air on the first animal part its last step went through: it stays in
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
        let hit = arrival(&spear, (victim, part, at));
        hits.write(hit);
        match (pierces(&hit.blow), roots.get(victim)) {
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

/// Gives a spear that came to rest back to the local robot when it stands within reach of it with
/// empty hands. A spear stuck in an animal stays there until [`super::pull::pull`] draws it out.
pub fn pick(
    mut commands: Commands,
    robot: Robot,
    spears: Query<(Entity, &Spear, &GlobalTransform, Option<&ChildOf>)>,
    (players, full, creatures): Reach,
    mut pack: Option<ResMut<Pack>>,
) {
    let Some((body, player)) = robot.pair() else {
        return;
    };
    let Ok(at) = players.get(player) else { return };
    if full.contains(body) {
        return;
    }
    let lying = |of: &Option<&ChildOf>| !of.is_some_and(|of| creatures.contains(of.parent()));
    let near = spears.iter().find(|(_, spear, placed, of)| {
        spear.rest != Rest::Flying
            && lying(of)
            && placed.translation().distance(at.translation) < PICK
    });
    if let Some((entity, spear, _, _)) = near {
        take(&mut commands, body, spear, pack.as_deref_mut());
        commands.entity(entity).despawn();
    }
}
