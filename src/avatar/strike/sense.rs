//! Sensing blows: after the transforms are propagated, the head of every swing in flight is swept
//! against the bounds of the parts of the animals, and the first part it meets takes the blow.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

use super::act::{Act, Strike, Timing};
use super::held::Held;
use super::hit::{Bounds, Hit, Tip, radius, sweet, touch};
use super::outcome::{Contact, Outcome, after, classify, rebound};
use super::path::Path;
use super::tune::{KICK, REACH_PAD};
use crate::craft::build::{Blow, Mode, blow, reference, strike as through};
use crate::fauna::Beast;
use crate::fauna::bleed::{refused, stiffness};
use crate::fauna::{Live, Part};
use crate::world::Terrain;

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

/// Returns the closest part of a creature among `parts` the path `was` → `now` meets with a head
/// padded by `pad` metres, with its animal and the point of contact.
pub fn first(
    parts: &Parts,
    creatures: &Query<(), Live>,
    (was, now): ([Vec3; 2], [Vec3; 2]),
    pad: f32,
) -> Option<(Entity, Entity, Vec3)> {
    parts
        .iter()
        .filter_map(|(part, placed, aabb, of)| {
            creatures.get(of.parent()).ok()?;
            let bounds = Bounds {
                centre: aabb.center.into(),
                half: aabb.half_extents.into(),
            };
            touch(was, now, placed, &bounds, pad).map(|at| (part, of.parent(), at))
        })
        .min_by(|a, b| {
            a.2.distance_squared(was[0])
                .total_cmp(&b.2.distance_squared(was[0]))
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
        outcome: Outcome::Clean,
    }
}

/// Run condition: some body is swinging.
pub fn swinging(strikes: Query<&Strike>) -> bool {
    strikes
        .iter()
        .any(|s| matches!(s.act, Act::Swing(_)) && !s.landed)
}

/// Returns what a swing came to on meeting `part` of `victim` at `at` with `blow` moving along `along`,
/// and how much the contact bounces: from the angle to the surface, whether the blow was blunt and
/// whether the outer layer of the animal refused its edge.
fn meet(
    (parts, beasts, terrain): (&Parts, &Query<&Beast>, &Terrain),
    (part, victim, at): (Entity, Entity, Vec3),
    (blow, along, tip): (&Blow, Vec3, f32),
) -> (Outcome, f32) {
    let middle = parts.get(part).map_or(at - along, |(_, placed, aabb, _)| {
        placed.transform_point(Vec3::from(aabb.center))
    });
    let normal = (at - middle).normalize_or(-along);
    let known = beasts.get(victim).ok().and_then(|beast| {
        let species = terrain
            .globe()
            .life()?
            .species
            .get(usize::from(beast.index))?;
        let index = beast.parts.iter().position(|p| *p == Some(part))?;
        Some((species, index))
    });
    let refused = known.is_some_and(|(species, index)| refused(species, index, blow));
    let contact = Contact {
        blunt: blow.mode == Mode::Blunt,
        refused,
        cos: along.dot(normal).abs(),
    };
    let struck = known.map_or(0.1, |(species, index)| stiffness(species, index));
    (classify(Some(contact)), rebound(struck, tip))
}

/// Sweeps every swing in flight that has landed nothing yet against the parts of the animals and
/// sends the first blow it lands. What the swing came to decides what the tool does next: a clean
/// blow that gives energy back stuns it for a moment, a blunt one that the body took stops it and
/// sets it back, a glance carries on.
pub fn sense(
    mut bodies: Query<(Entity, &GlobalTransform, &mut Strike, &Held)>,
    (parts, creatures, beasts): (Parts, Query<(), Live>, Query<&Beast>),
    terrain: Res<Terrain>,
    mut hits: MessageWriter<Hit>,
) {
    for (body, placed, mut strike, held) in &mut bodies {
        let Act::Swing(t) = strike.act else { continue };
        if strike.landed {
            continue;
        }
        let world = |pair: [Vec3; 2]| pair.map(|p| placed.transform_point(p));
        let (now, was) = (world(strike.blade[0]), world(strike.blade[1]));
        let pad = (held.build.head.size() / 2.0).max(REACH_PAD);
        let Some((part, victim, at)) = first(&parts, &creatures, (was, now), pad) else {
            continue;
        };
        let (blow, stun) = landed(held, now, at);
        let along = (now[1] - was[1]).normalize_or_zero();
        let tip = held.build.head.material.stiffness;
        let met = meet(
            (&parts, &beasts, &terrain),
            (part, victim, at),
            (&blow, along, tip),
        );
        let timing = Timing::of(held, &Path::of(held.style, strike.dir)).felt(&strike.hurt);
        strike.landed = true;
        strike.result = met.0;
        if let Some(act) = after(met.0, stun, t / timing.swing.max(f32::EPSILON), met.1) {
            strike.act = act;
        }
        let mut sent = hit(held, body, (victim, part, at), (blow, along));
        sent.outcome = met.0;
        hits.write(sent);
    }
}
