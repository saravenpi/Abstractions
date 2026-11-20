//! Tests of harm: a small blow hurts and a big one kills, goes into the ledger, leaves a carcass and
//! makes the body fall.

use bevy::prelude::*;

use super::harm::{Dead, harm};
use super::slump::Slump;
use crate::avatar::strike::{Hit, Tip};
use crate::craft::build::{Blow, Intent, Mode};
use crate::craft::harvest::Harvest;
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::fauna::rig::app;
use crate::fauna::stage::run;
use crate::fauna::want::Lead;

/// Returns a blow of `energy` joules on `victim`.
fn hit(victim: Entity, energy: f32) -> Hit {
    Hit {
        attacker: Entity::PLACEHOLDER,
        item: 0,
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
        tip: Tip {
            stiffness: 30.0,
            mass: 0.3,
        },
    }
}

/// Asserts that `victim` is dead, falling, led by its carcass, and the only carcass there is.
fn lies(app: &App, victim: Entity) {
    let body = app.world().entity(victim);
    assert!(body.contains::<Dead>() && body.contains::<Slump>());
    assert!(matches!(
        body.get::<Beast>().expect("kept").lead,
        Lead::Carcass { .. }
    ));
    assert_eq!(app.world().resource::<Herds>().slain.len(), 1);
}

/// A blow takes its energy from the health, and one past the health kills, records the death, leaves
/// one carcass and marks the animal dead, once.
#[test]
fn a_blow_hurts_and_a_big_one_kills() {
    let (mut app, herd) = app(true);
    assert!(herd.is_some());
    app.add_message::<Hit>()
        .add_message::<Harvest>()
        .add_systems(Update, harm);
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
    lies(&app, victim);
    let Lead::Herd { id, slot } = lead else {
        return;
    };
    let now = app.world().resource::<crate::space::Clock>().seconds;
    assert!(app.world().resource::<Herds>().ledger.dead(id, slot, now));
}
