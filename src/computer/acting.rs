//! The action line of the course on the map: one button that sets the course to the selection,
//! engages the autopilot or clears the course, with its key, or words that say how far along
//! the route is or why it cannot start.

use bevy::prelude::*;

use super::board::Board;
use super::course::{Act, Offer, offer};
use super::pointer::{MapView, selected};
use super::route::{Destination, RouteStatus, Target};
use super::view::{Action, ActionSlot};
use crate::menus::key_name;
use crate::settings::{Persistent, Settings};
use crate::ui::{Tone, action};
use crate::universe::Universe;

/// Returns the name of the key that runs `act`.
fn key_of(act: Act, engage: KeyCode) -> String {
    match act {
        Act::Set => "Enter".to_string(),
        Act::Engage => key_name(engage),
        Act::Clear => "Delete".to_string(),
        Act::Home => "Home".to_string(),
    }
}

/// The room of the button and the words of the action line.
type Lines<'w, 's> = (
    Query<'w, 's, Entity, With<ActionSlot>>,
    Query<'w, 's, &'static mut Text, With<Action>>,
);

/// What the action line is worked out from.
type Course<'w> = (
    Res<'w, Board>,
    Res<'w, MapView>,
    Res<'w, Destination>,
    Res<'w, RouteStatus>,
);

/// Puts the button of `wanted` in the room `slot`, or its words on the action line.
fn show(commands: &mut Commands, slot: Entity, wanted: &Offer, engage: KeyCode, words: &mut Lines) {
    commands.entity(slot).despawn_children();
    let said = match wanted {
        Offer::Button(act, icon, label) => {
            let key = key_of(*act, engage);
            let button = (*act, action(*icon, label, &key, Tone::Accent));
            commands.entity(slot).with_child(button);
            String::new()
        }
        Offer::Words(said) => said.clone(),
    };
    for mut text in &mut words.1 {
        text.set_if_neq(Text(said.clone()));
    }
}

/// What the action line showed last: its offer, the selection it was for, and its room.
type Shown = Option<(Offer, Option<Target>, Entity)>;

/// Writes the action line when the selection, the course or the route changes: the button in
/// its room, or the words alone. Nothing selected, it offers no course to set.
pub(super) fn write(
    (board, view, destination, route): Course,
    (settings, universe): (Res<Persistent<Settings>>, Res<Universe>),
    mut commands: Commands,
    mut lines: Lines,
    mut last: Local<Shown>,
) {
    let Some(slot) = lines.0.iter().next() else {
        return;
    };
    let chosen = selected(&board, &view).map(|row| board.lines[row].target);
    let moved = board.is_changed() || destination.is_changed() || route.is_changed();
    let kept = last
        .as_ref()
        .is_some_and(|(_, was, at)| *at == slot && *was == chosen);
    if !moved && kept {
        return;
    }
    let set = chosen.is_some() && chosen != destination.0;
    let name = destination.0.map(|target| target.name(&universe));
    let wanted = offer(name.as_deref(), set, *route);
    if last.as_ref() == Some(&(wanted.clone(), chosen, slot)) {
        return;
    }
    show(
        &mut commands,
        slot,
        &wanted,
        settings.controls.keys.engage,
        &mut lines,
    );
    *last = Some((wanted, chosen, slot));
}
