//! The action line of the course on the map: the engage key and what it does while the
//! autopilot can take the course, or why it cannot, or how far along the route is.

use bevy::prelude::*;

use super::route::{RouteStatus, Target};
use super::scene::Scene;
use super::view::{Action, ActionKey, Crumbs, Status};
use crate::menus::key_name;

/// The action line of the course: its words and its key.
pub(super) type Acting<'w, 's> = (
    Query<
        'w,
        's,
        &'static mut Text,
        (
            With<Action>,
            Without<ActionKey>,
            Without<Status>,
            Without<Crumbs>,
        ),
    >,
    Query<
        'w,
        's,
        &'static mut Text,
        (
            With<ActionKey>,
            Without<Action>,
            Without<Status>,
            Without<Crumbs>,
        ),
    >,
);

/// Writes the action line of the course, its key only while the key engages the autopilot.
pub(super) fn act(
    scene: &Scene,
    (destination, route): (Option<Target>, RouteStatus),
    key: KeyCode,
    (mut words, mut keys): Acting,
) {
    let named = destination.map(|target| target.name(&scene.universe));
    let (engage, said) = named.map_or((false, String::new()), |name| {
        super::course::action(&name, route)
    });
    let shown = if engage { key_name(key) } else { String::new() };
    for mut text in &mut words {
        text.set_if_neq(Text(said.clone()));
    }
    for mut text in &mut keys {
        text.set_if_neq(Text(shown.clone()));
    }
}
