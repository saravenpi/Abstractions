//! The intents the keys and the mouse of this machine send on the wings.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::eyes::gliding;
use crate::player::eyes::STEER;
use crate::player::keys::send;
use crate::settings::{Persistent, Settings};
use crate::sim::{Intent, IntentOf};
use crate::view::{Heading, ViewMode, ViewPrefs};

/// Runs the sending of the intents once in `world` and returns what was sent.
fn sent(world: &mut World) -> Vec<Intent> {
    world.resource_mut::<Messages<IntentOf>>().clear();
    world.run_system_once(send).expect("the intents are sent");
    let mut out = world.resource_mut::<Messages<IntentOf>>();
    out.drain().map(|message| message.intent).collect()
}

/// Returns a gliding world whose player looks from the eyes.
fn first_person() -> World {
    let mut world = gliding();
    let on_foot = ViewMode::First;
    world.insert_resource(ViewPrefs {
        on_foot,
        ..default()
    });
    world
}

/// The look the local player sends is the view on the wings as on foot. The steer button
/// sends the steer intent and the turn keys of the ship send the yaw of the wings.
#[test]
fn the_look_is_the_view_and_the_steer_and_yaw_keys_are_sent() {
    let mut world = first_person();
    world.init_resource::<Messages<IntentOf>>();
    *world.resource_mut::<Heading>() = Heading {
        yaw: 1.0,
        pitch: 0.3,
    };
    let look = Intent::Look {
        yaw: 1.0,
        pitch: 0.3,
        turn: 0.0,
        behind: false,
    };
    let first = sent(&mut world);
    assert!(
        first.contains(&look) && !first.contains(&Intent::Steer),
        "{first:?}"
    );
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(STEER);
    let keys = world.resource::<Persistent<Settings>>().controls.keys;
    world
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(keys.ship.turn_right);
    let held = sent(&mut world);
    assert!(
        held.contains(&Intent::Steer) && held.contains(&Intent::Yaw(1.0)),
        "{held:?}"
    );
}
