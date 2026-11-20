//! The path over the map: the levels from the universe down to the one shown, each a button
//! that jumps to its level but the last. It is the only way to change level besides a double
//! click into a place. Backspace goes up one level.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::Place;
use super::go::Nav;
use crate::computer::Typing;
use crate::ui::{Tone, frame, label, theme};
use crate::universe::Universe;

/// Marks the row the parts of the path are written in.
#[derive(Component)]
pub struct Parts;

/// A part of the path, which jumps to its level.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crumb(pub Place);

/// The node of a small button of the path.
fn button_node() -> Node {
    Node {
        align_items: AlignItems::Center,
        padding: UiRect::axes(px(theme::GAP_XS), px(theme::GAP_XS)),
        ..default()
    }
}

/// The path over the map, empty until [`write`] fills it. It keeps its height while empty.
pub fn path() -> impl Bundle {
    (
        Parts,
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::GAP_XS),
            min_height: px(theme::ROW_COMPACT),
            ..default()
        },
    )
}

/// Returns one part of the path: a button for a level above the one shown, plain words for it.
fn part(commands: &mut ChildSpawnerCommands, place: Place, name: String, last: bool) {
    if last {
        commands.spawn(label(name, theme::FONT_SMALL, Tone::Plain));
        return;
    }
    let words = label(name, theme::FONT_SMALL, Tone::Muted);
    commands.spawn((Crumb(place), frame(), button_node(), children![words]));
    commands.spawn(label(">", theme::FONT_SMALL, Tone::Muted));
}

/// Writes the path again when the level shown changes.
pub(super) fn write(
    (nav, universe): (Nav, Option<Res<Universe>>),
    rows: Query<Entity, With<Parts>>,
    mut commands: Commands,
    mut last: Local<Option<(Place, Entity)>>,
) {
    let (Some(universe), Some(row)) = (universe, rows.iter().next()) else {
        return;
    };
    let here = nav.here().place;
    if *last == Some((here, row)) {
        return;
    }
    *last = Some((here, row));
    let path = here.path();
    commands
        .entity(row)
        .despawn_children()
        .with_children(|row| {
            for (at, place) in path.iter().enumerate() {
                part(row, *place, place.name(&universe), at + 1 == path.len());
            }
        });
}

/// Goes back on Backspace, unless something is typed.
pub(super) fn back_key(keys: Res<ButtonInput<KeyCode>>, typing: Res<Typing>, mut nav: Nav) {
    if keys.just_pressed(KeyCode::Backspace) && typing.is_off() {
        nav.back();
    }
}

/// Goes to the level of a part of the path clicked.
pub(super) fn on_press(event: On<Activate>, buttons: Query<&Crumb>, mut nav: Nav) {
    if let Ok(Crumb(place)) = buttons.get(event.entity) {
        nav.enter(*place);
    }
}
