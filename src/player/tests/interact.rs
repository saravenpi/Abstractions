//! Tests of the interact key: one press does one thing, the action the prompt names.

use bevy::prelude::*;

use crate::player::{Interact, Interacting, Offer, Verb};

/// Returns an app that settles the offers each frame, with a keyboard.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins(Interacting)
        .init_resource::<ButtonInput<KeyCode>>();
    app
}

/// Offers each of `verbs` this frame and runs the frame after, where the offers are settled.
fn offer(app: &mut App, verbs: &[Verb]) {
    let mut interact = app.world_mut().resource_mut::<Interact>();
    verbs.iter().for_each(|verb| interact.offer(*verb, "it"));
    app.update();
}

/// With a spear in a carcass beside the ship, the key pulls the spear and does nothing else; with
/// the spear out it cuts; with nothing to cut it boards. Each press fires exactly one action.
#[test]
fn one_press_does_the_one_thing_the_prompt_names() {
    let mut app = app();
    let key = KeyCode::KeyF;
    let shared = [Verb::Pull, Verb::Carve, Verb::Board, Verb::Leave];
    for (offered, wanted) in [
        (&shared[..], Verb::Pull),
        (&shared[1..], Verb::Carve),
        (&shared[2..], Verb::Board),
        (&[Verb::Leave, Verb::Stone, Verb::Mend][..], Verb::Leave),
    ] {
        offer(&mut app, offered);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        let interact = app.world().resource::<Interact>();
        let keys = app.world().resource::<ButtonInput<KeyCode>>();
        let fired: Vec<Verb> = shared
            .into_iter()
            .filter(|verb| interact.pressed(*verb, keys, key))
            .collect();
        assert_eq!(fired, [wanted], "offered {offered:?}");
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
    }
}

/// An action that is no longer offered leaves the prompt on the next frame, and the key then does
/// nothing; the name of what the action acts on reaches the prompt.
#[test]
fn an_offer_lasts_one_frame_and_keeps_its_name() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<Interact>()
        .offer(Verb::Carve, "thigh");
    app.update();
    let chosen = app.world().resource::<Interact>().chosen();
    let thigh = Offer {
        verb: Verb::Carve,
        thing: "thigh",
    };
    assert_eq!(chosen, Some(thigh));
    app.update();
    let interact = app.world().resource::<Interact>();
    assert_eq!(interact.chosen(), None);
    assert!(!interact.is(Verb::Carve));
}
