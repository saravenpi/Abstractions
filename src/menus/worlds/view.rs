//! Static layout of the worlds menu.

use bevy::input_focus::{AutoFocus, InputFocus};
use bevy::prelude::*;

use super::{EXTRA_SLOT, Pending, Row, Scroll, Selected, WorldList};
use crate::menus::slots::Slot;
use crate::menus::{Menu, Nav};
use crate::saves::{SaveDir, list};
use crate::ui::{Tone, caps, column, footer, item, root, theme};

/// Marks the column that holds the world rows.
#[derive(Component)]
pub(super) struct RowsBox;

/// Marks the key hint line.
#[derive(Component)]
pub(super) struct Hint;

/// Hint shown while a row has focus.
const ROW_HINT: &str = "Enter play, Delete remove, Esc back";

/// Hint shown otherwise.
const IDLE_HINT: &str = "Esc to go back";

/// The New world and Back items below the rows.
fn extras() -> impl Bundle {
    (
        Node {
            width: px(theme::MENU_WIDTH),
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(px(theme::SPACE * 2.0)),
            ..default()
        },
        children![
            (
                item("New world", Tone::Plain),
                Nav::To(Menu::NewGame),
                Slot(EXTRA_SLOT),
                AutoFocus
            ),
            (
                item("Back", Tone::Plain),
                Nav::To(Menu::Main),
                Slot(EXTRA_SLOT + 1)
            ),
        ],
    )
}

/// Loads the worlds and spawns the menu frame. The rows are built by `rows::rebuild`.
pub(super) fn spawn(mut commands: Commands, dir: Res<SaveDir>, mut worlds: ResMut<WorldList>) {
    worlds.0 = list(&dir.0);
    commands.insert_resource(Pending(None));
    commands.insert_resource(Selected(0));
    commands.insert_resource(Scroll(0));
    commands.spawn((
        DespawnOnExit(Menu::Worlds),
        root(),
        children![
            (
                caps("Worlds"),
                Node {
                    margin: UiRect::bottom(px(theme::SPACE * 2.0)),
                    ..default()
                }
            ),
            (RowsBox, column(px(theme::MENU_WIDTH * 1.6), 0.0)),
            extras(),
            (Hint, footer(IDLE_HINT)),
        ],
    ));
}

/// Shows the key hints of the focused row, or the idle hint.
pub(super) fn show_hint(
    focus: Res<InputFocus>,
    rows: Query<(), With<Row>>,
    mut hints: Query<&mut Text, With<Hint>>,
) {
    let on_row = focus.get().is_some_and(|entity| rows.contains(entity));
    let shown = if on_row { ROW_HINT } else { IDLE_HINT };
    for mut text in &mut hints {
        text.set_if_neq(Text(shown.to_string()));
    }
}
