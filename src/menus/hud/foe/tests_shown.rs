//! Tests of the layer of the strips: on foot with no tool drawn, a strip that shows an animal is
//! visible on screen, every node from it up to its root shown, under its own gate and not under
//! the reticle's.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::super::context::{Context, Element, Where};
use super::super::fade::{Gate, step};
use super::{Foe, Strips, draw, layer};

/// Shows a strip in slot 1 over an animal looked at, as the strips do with nothing in the hands.
fn looked_at(mut strips: Strips) {
    draw(&mut strips, 1, Some((0.5, "grazer", (-64.0, 40.0))), 1.0);
}

/// Returns an app on foot with the layer of the strips, its gate opening with the context and a
/// strip drawn every frame.
fn app() -> App {
    let mut app = App::new();
    let frame = TimeUpdateStrategy::ManualDuration(Duration::from_millis(50));
    app.add_plugins(MinimalPlugins)
        .insert_resource(frame)
        .insert_resource(Where(Context::OnFoot))
        .add_systems(Update, (looked_at, step).chain());
    app.world_mut().spawn(layer());
    app
}

/// Returns true when `entity` and every node above it are shown: none hidden, none laid out as
/// nothing.
fn shown(world: &World, entity: Entity) -> bool {
    let mut at = Some(entity);
    while let Some(now) = at {
        let hidden = world.get::<Visibility>(now) == Some(&Visibility::Hidden);
        let none = world
            .get::<Node>(now)
            .is_some_and(|node| node.display == Display::None);
        if hidden || none {
            return false;
        }
        at = world.get::<ChildOf>(now).map(ChildOf::parent);
    }
    true
}

/// With no tool drawn and the reticle nowhere, the strip of an animal looked at is visible once its
/// gate opened, and the gate above it is the one of the labels.
#[test]
fn a_looked_at_animal_shows_its_strip_with_empty_hands() {
    let mut app = app();
    for _ in 0..20 {
        app.update();
    }
    let world = app.world_mut();
    let strip = world
        .query::<(Entity, &Foe)>()
        .iter(world)
        .find(|(_, foe)| foe.0 == 1)
        .map(|(entity, _)| entity)
        .expect("a strip in slot 1");
    let gates: Vec<Element> = world
        .query::<&Gate>()
        .iter(world)
        .map(|g| g.element)
        .collect();
    assert_eq!(gates, [Element::Labels]);
    assert!(shown(world, strip), "the strip is hidden");
}
