//! Sensing blows: after the transforms are propagated, the head of every swing in flight is swept
//! against the bounds of the parts of the animals, and the first part it meets takes the blow.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

use super::act::{Act, Strike, Timing};
use super::held::Held;
use super::hit::{Bounds, Hit, Tip, radius, sweet, touch};
use super::path::Path;
use super::tune::KICK;
use crate::craft::build::{Blow, blow, reference, strike as through};
use crate::fauna::{Live, Part};

/// The parts of animals as the sweep sees them: placed, bounded, and the animal they belong to.
pub type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        &'static Aabb,
        &'static ChildOf,
    ),
    With<Part>,
>;

/// Returns the first part of a creature among `parts` the path `was` → `now` meets with a head
/// padded by `pad` metres, with its animal and the point of contact.
pub fn first(
    parts: &Parts,
    creatures: &Query<(), Live>,
    (was, now): ([Vec3; 2], [Vec3; 2]),
    pad: f32,
) -> Option<(Entity, Entity, Vec3)> {
    parts.iter().find_map(|(part, placed, aabb, of)| {
        creatures.get(of.parent()).ok()?;
        let bounds = Bounds {
            centre: aabb.center.into(),
            half: aabb.half_extents.into(),
        };
        touch(was, now, placed, &bounds, pad).map(|at| (part, of.parent(), at))
    })
}

/// Returns the blow `held` lands at `at` with the blade `now`, its energy taken at the contact
/// radius times the sweet spot, and the kickback stun: 0.1 s × the share of the energy the
/// reference target gave back, less what a counterweight calms.
pub fn landed(held: &Held, now: [Vec3; 2], at: Vec3) -> (Blow, f32) {
    let mut out = blow(&held.build, held.style.intent(), &held.place);
    out.energy *= sweet(radius(now[0], now[1], at), held.build.shaft.length);
    let kept = through(&out, &reference()).left / out.energy.max(f32::EPSILON);
    (out, KICK * kept.clamp(0.0, 1.0) * (1.0 - held.counter()))
}

/// Returns the hit of the tool `held` by `body` on `part` of `victim` at `at`, moving along `along`.
fn hit(
    held: &Held,
    body: Entity,
    (victim, part, at): (Entity, Entity, Vec3),
    (blow, along): (Blow, Vec3),
) -> Hit {
    Hit {
        attacker: body,
        item: held.id,
        victim,
        part,
        at,
        along,
        blow,
        intent: held.style.intent(),
        tip: Tip {
            stiffness: held.build.head.material.stiffness,
            mass: held.build.head.mass,
        },
    }
}

/// Run condition: some body is swinging.
pub fn swinging(strikes: Query<&Strike>) -> bool {
    strikes
        .iter()
        .any(|s| matches!(s.act, Act::Swing(_)) && !s.landed)
}

/// Sweeps every swing in flight that has landed nothing yet against the parts of the animals and
/// sends the first blow it lands; a blow that gives energy back stuns the swing for a moment.
pub fn sense(
    mut bodies: Query<(Entity, &GlobalTransform, &mut Strike, &Held)>,
    (parts, creatures): (Parts, Query<(), Live>),
    mut hits: MessageWriter<Hit>,
) {
    for (body, placed, mut strike, held) in &mut bodies {
        let Act::Swing(t) = strike.act else { continue };
        if strike.landed {
            continue;
        }
        let world = |pair: [Vec3; 2]| pair.map(|p| placed.transform_point(p));
        let (now, was) = (world(strike.blade[0]), world(strike.blade[1]));
        let pad = held.build.head.size() / 2.0;
        let Some((part, victim, at)) = first(&parts, &creatures, (was, now), pad) else {
            continue;
        };
        let (blow, stun) = landed(held, now, at);
        let timing = Timing::of(held, &Path::of(held.style, strike.dir)).felt(&strike.hurt);
        strike.landed = true;
        if stun > 0.0 {
            strike.act = Act::Stun(stun, t / timing.swing.max(f32::EPSILON));
        }
        let along = (now[1] - was[1]).normalize_or_zero();
        hits.write(hit(held, body, (victim, part, at), (blow, along)));
    }
}
