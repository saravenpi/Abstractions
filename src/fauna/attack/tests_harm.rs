//! Tests of harm: a small blow hurts and a big one kills and goes into the ledger.

use bevy::prelude::*;

use super::harm::{Dead, harm};
use crate::avatar::strike::Hit;
use crate::craft::build::{Blow, Intent, Mode};
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::fauna::rig::app;
use crate::fauna::stage::run;
use crate::fauna::want::Lead;

/// Returns a blow of `energy` joules on `victim`.
fn hit(victim: Entity, energy: f32) -> Hit {
    Hit {
        attacker: Entity::PLACEHOLDER,
        victim,
        part: Entity::PLACEHOLDER,
        at: Vec3::ZERO,
        along: Vec3::X,
        blow: Blow {
            energy,
            mode: Mode::Slash,
            width: 0.01,
            area: 0.01,
            hardness: 5.0,
        },
        intent: Intent::Cut,
    }
}

/// A blow takes its energy from the health, and one past the health kills, records the death and
/// marks the animal dead, once.
#[test]
fn a_blow_hurts_and_a_big_one_kills() {
    let (mut app, herd) = app(true);
    assert!(herd.is_some());
    app.add_message::<Hit>().add_systems(Update, harm);
    run(&mut app, 3.0, 1.0 / 30.0);
    let world = app.world_mut();
    let mut herds = world.query::<(Entity, &Beast)>();
    let (victim, before, lead) = herds
        .iter(world)
        .find(|(_, b)| matches!(b.lead, Lead::Herd { .. }))
        .map(|(e, b)| (e, b.wits.health, b.lead))
        .expect("an animal of a herd is drawn");
    world.write_message(hit(victim, before.max * 0.25));
    run(&mut app, 0.0, 1.0);
    app.world_mut().run_schedule(Update);
    let after = app.world().get::<Beast>(victim).expect("alive").wits.health;
    assert!(after.now < before.now && !app.world().entity(victim).contains::<Dead>());
    app.world_mut().write_message(hit(victim, before.max));
    app.world_mut().run_schedule(Update);
    assert!(app.world().entity(victim).contains::<Dead>());
    let Lead::Herd { id, slot } = lead else {
        return;
    };
    let now = app.world().resource::<crate::space::Clock>().seconds;
    assert!(app.world().resource::<Herds>().ledger.dead(id, slot, now));
}
