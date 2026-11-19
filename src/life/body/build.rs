//! Building a body from its plan, archetype, role, history and world.

use super::super::{Guild, Habitat, History, Plan, Role, World};
use super::frame::{frame, length};
use super::kit::{Kit, MAX_PARTS};
use super::make::Make;
use super::rules::traits;
use super::{Body, Locomotion, crystal, radial, segmented, skeletal, soft};
use crate::universe::hash::{Rng, child};

/// Salt of the dice of a body, apart from the other uses of the species seed.
const SALT: u64 = 0xB0D7;

/// Returns the number of archetypes of `plan`. Archetype numbers wrap around it.
pub fn archetypes(plan: Plan) -> u8 {
    match plan {
        Plan::Skeletal => skeletal::COUNT,
        Plan::Segmented => segmented::COUNT,
        Plan::Radial => radial::COUNT,
        Plan::Amorphous => soft::AMORPHOUS,
        Plan::Colonial => soft::COLONIAL,
        Plan::Crystalline => crystal::COUNT,
    }
}

/// Returns the body of a species whose `form` is its plan and archetype, with `role` and `history` on `world`.
///
/// A producer has an empty body: flora draws it. Every other body follows the rule table in `rules`,
/// keeps its parts parents first and mirrors every part off the centre line.
pub fn build(seed: u64, form: (Plan, u8), role: &Role, history: &History, world: &World) -> Body {
    let (plan, archetype) = form;
    if role.guild == Guild::Producer {
        return finish(Kit::new(0).body, plan, role.habitat);
    }
    let crawler = plan == Plan::Segmented && archetype % segmented::COUNT == 0;
    let cap = if crawler {
        segmented::CRAWLER_PARTS
    } else {
        MAX_PARTS
    };
    let mut m = Make {
        kit: Kit::new(cap),
        t: traits(role, history, world),
        guild: role.guild,
        habitat: role.habitat,
        rng: Rng::new(child(seed, SALT)),
    };
    match plan {
        Plan::Skeletal => skeletal::build(&mut m, archetype),
        Plan::Segmented => segmented::build(&mut m, archetype),
        Plan::Radial => radial::build(&mut m, archetype),
        Plan::Amorphous => soft::amorphous(&mut m, archetype),
        Plan::Colonial => soft::colonial(&mut m, archetype),
        Plan::Crystalline => crystal::build(&mut m, archetype),
    }
    finish(m.kit.body, plan, role.habitat)
}

/// Returns `body` with its motion model, its length and its hip height filled in.
fn finish(mut body: Body, plan: Plan, habitat: Habitat) -> Body {
    body.locomotion = locomotion(&body, plan, habitat);
    body.length = length(&body.parts);
    body.height = match body.legs.first() {
        Some(leg) => frame(&body.parts, usize::from(leg.bones[0])).0.y,
        None => body.parts.first().map_or(0.0, |p| p.socket.y),
    };
    body
}

/// Returns the motion model of `body`: wings fly, then the plan decides, then water swims and legless
/// bodies slither.
fn locomotion(body: &Body, plan: Plan, habitat: Habitat) -> Locomotion {
    match plan {
        _ if body.wings.is_some() => Locomotion::Fly,
        Plan::Radial if body.legs.is_empty() => Locomotion::Pulse,
        Plan::Amorphous => Locomotion::Ooze,
        Plan::Colonial => Locomotion::Swarm,
        Plan::Crystalline => Locomotion::Tumble,
        _ if habitat == Habitat::Water => Locomotion::Swim,
        _ if body.legs.is_empty() => Locomotion::Slither,
        _ => Locomotion::Walk,
    }
}
