//! The keys that open the wings: the jump pressed again in the air spreads them whatever else
//! is held, a run held since the ground never fires the boots on them, and every change of the
//! wings is told as a [`Spread`] message.

use bevy::prelude::*;

use super::wings::{hold, lift, world};
use crate::gear::Mode;
use crate::player::{Player, Spread};
use crate::sim::Intent;

/// The forward key held.
const AHEAD: Intent = Intent::Move {
    forward: 1.0,
    strafe: 0.0,
};

/// A sprint off a ledge, the run still held, and the jump pressed again in the air: the wings
/// spread into a glide and the boots stay off, so the run that carried the body over the edge
/// never turns the glide into powered flight. The run pressed again on the wings boosts.
#[test]
fn a_sprint_off_a_ledge_glides_and_a_new_run_press_boosts() {
    let mut app = world();
    hold(&mut app, 30, &[AHEAD, Intent::Boost]);
    lift(&mut app);
    let (over, _, fired) = hold(&mut app, 6, &[AHEAD, Intent::Boost]);
    assert!(!over.grounded && !over.wings && !fired, "{over:?}");
    hold(&mut app, 2, &[AHEAD, Intent::Boost, Intent::Jump]);
    let (glide, gear, fired) = hold(&mut app, 30, &[AHEAD, Intent::Boost]);
    assert!(glide.wings && !glide.boots && !fired, "{glide:?}");
    assert_eq!(gear.state.mode, Mode::Glide);
    hold(&mut app, 1, &[AHEAD]);
    let (fly, gear, _) = hold(&mut app, 10, &[AHEAD, Intent::Boost]);
    assert!(fly.wings && fly.boots, "{fly:?}");
    assert_eq!(gear.state.mode, Mode::Fly);
}

/// Returns the [`Spread`] messages sent since the last call.
fn told(app: &mut App) -> Vec<Spread> {
    app.world_mut()
        .resource_mut::<Messages<Spread>>()
        .drain()
        .collect()
}

/// The press that opens the wings tells it once, with the body, on that frame; the press that
/// folds them tells that once too; nothing is told while they stay as they are.
#[test]
fn every_change_of_the_wings_is_told_once() {
    let mut app = world();
    app.add_message::<Spread>();
    lift(&mut app);
    hold(&mut app, 6, &[]);
    assert!(told(&mut app).is_empty());
    let (open, _, _) = hold(&mut app, 1, &[Intent::Jump]);
    let world = app.world_mut();
    let mut players = world.query_filtered::<Entity, With<Player>>();
    let entity = players.iter(world).next().expect("the body");
    assert!(open.wings);
    assert_eq!(
        told(&mut app),
        vec![Spread {
            body: entity,
            open: true
        }]
    );
    hold(&mut app, 20, &[]);
    assert!(told(&mut app).is_empty());
    hold(&mut app, 1, &[Intent::Jump]);
    assert_eq!(
        told(&mut app),
        vec![Spread {
            body: entity,
            open: false
        }]
    );
}
