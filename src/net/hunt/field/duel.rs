//! The blows of this robot on the robots of the others: a swing that enters the body of a puppet
//! lands a blow on its player, who hurts and is pushed by it as by an animal. The puppet staggers on
//! this screen at once, from the same push, and the swing is heard by everyone.

use abstractions_net::log::PlayerId;
use abstractions_net::wire::Blow as Told;
use bevy::math::Vec2;
use bevy::prelude::*;

use super::codec::struck_of;
use super::hits::upright;
use super::tune::BODY_RADIUS;
use crate::audio::{Body, Impact, Outer};
use crate::avatar::hurt::{Reel, capacity, impulse, reel};
use crate::avatar::strike::{Act, Held, Strike, landed};
use crate::craft::build::{Mode, mass};
use crate::fauna::senses::tune::ROBOT_KG;
use crate::net::hunt::hooks;
use crate::net::link::Link;
use crate::net::state::Remote;
use crate::net::state::Remotes;
use crate::space::Anchor;

/// Returns the point of the blade `now` (its grip and its tip) nearest the middle `centre` of a body,
/// when it is within [`BODY_RADIUS`] of it.
pub fn reach(now: [Vec3; 2], centre: Vec3) -> Option<Vec3> {
    let blade = now[1] - now[0];
    let share = (centre - now[0]).dot(blade) / blade.length_squared().max(f32::EPSILON);
    let nearest = now[0] + blade * share.clamp(0.0, 1.0);
    (nearest.distance(centre) < BODY_RADIUS).then_some(nearest)
}

/// Returns the puppet of a robot the blade `now` reaches, with its player, its place and the point
/// it reaches.
fn struck(
    (remotes, puppets): (&Remotes, &Query<&GlobalTransform, With<Remote>>),
    now: [Vec3; 2],
) -> Option<(PlayerId, Entity, GlobalTransform, Vec3)> {
    remotes.robots.iter().find_map(|(player, track)| {
        let puppet = track.puppet?;
        let body = *puppets.get(puppet).ok()?;
        Some((*player, puppet, body, reach(now, body.translation())?))
    })
}

/// Returns the sound of the blade of `held` on the robot plate at `at`, as the blow `told` carries.
fn ring(held: &Held, told: &Told, at: Vec3) -> Impact {
    let head = &held.build.head;
    let outer = Outer::RobotPlate;
    Impact {
        at,
        joules: told.joules,
        cut: told.cut,
        outer,
        struck: struck_of(outer),
        tool: Body {
            stiffness: head.material.stiffness,
            mass: head.mass,
        },
        deflected: false,
    }
}

/// Returns the blow of `held` that reaches `at` with the blade `now`, going `along`, on `victim`.
fn blow_of(
    held: &Held,
    (now, along, at): ([Vec3; 2], Vec3, Vec3),
    (victim, anchor, body): (PlayerId, &Anchor, &GlobalTransform),
) -> Told {
    let (hit, _) = landed(held, now, at);
    let turn = upright(anchor, body.translation()).inverse();
    let tool = mass(&held.build.pieces()).max(f32::EPSILON);
    Told {
        victim,
        at: (turn * (at - body.translation())).to_array(),
        joules: hit.energy,
        cut: hit.mode != Mode::Blunt,
        dir: (turn * along).to_array(),
        momentum: (2.0 * tool * hit.energy).sqrt(),
    }
}

/// Returns the stagger the blow `told` gives the puppet at `body`.
fn stagger(told: &Told, body: &GlobalTransform) -> Option<Reel> {
    let local = body.rotation().inverse() * Vec3::from(told.dir);
    let speed = impulse(told.momentum, ROBOT_KG);
    reel(speed, capacity(0.0, false), Vec2::new(local.x, -local.z))
}

/// Lands the swing of every robot of this machine that enters the body of another player's robot.
pub fn duel(
    (mut link, mut sounds): (ResMut<Link>, MessageWriter<Impact>),
    (anchor, remotes): (Res<Anchor>, Res<Remotes>),
    mut bodies: Query<(&GlobalTransform, &mut Strike, &Held), Without<Remote>>,
    puppets: Query<&GlobalTransform, With<Remote>>,
    mut commands: Commands,
) {
    for (placed, mut strike, held) in &mut bodies {
        if !matches!(strike.act, Act::Swing(_)) || strike.landed {
            continue;
        }
        let now = strike.blade[0].map(|p| placed.transform_point(p));
        let Some((victim, puppet, body, at)) = struck((&remotes, &puppets), now) else {
            continue;
        };
        strike.landed = true;
        let along = (now[1] - strike.blade[1][1]).normalize_or_zero();
        let told = blow_of(held, (now, along, at), (victim, &anchor, &body));
        sounds.write(ring(held, &told, at));
        if let Some(reel) = stagger(&told, &body) {
            commands.entity(puppet).insert(reel);
        }
        hooks::send_blow(&mut link, told);
    }
}
