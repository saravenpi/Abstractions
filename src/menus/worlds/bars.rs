//! The bars of actions of the worlds menu: one for each mode, each action with its icon, its
//! label and its key.

use bevy::prelude::*;

use super::Mode;
use super::act::Act;
use crate::menus::{Menu, Nav};
use crate::ui::{Cancels, Icon, Selected, Tone, action, column, strip};

/// A bar of actions, shown in one mode only.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Bar {
    /// Actions on the selected world.
    Browse,
    /// Applying or dropping the typed name.
    Rename,
    /// Keeping or deleting the world.
    Delete,
}

impl Bar {
    /// Returns whether the bar is drawn in `mode`. Without a world only the bar that browses
    /// is, and its actions on a world do nothing.
    pub(super) fn shown(self, mode: &Mode, empty: bool) -> bool {
        match (self, mode) {
            (Bar::Browse, Mode::Browse) => true,
            (Bar::Rename, Mode::Rename(_)) => !empty,
            (Bar::Delete, Mode::Delete { .. }) => !empty,
            _ => false,
        }
    }
}

/// One of the two answers to the delete question. True deletes, false keeps.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Choice(pub bool);

/// The actions on the selected world, creating a world and going back.
fn browse() -> impl Bundle {
    let new_world = action(Icon::Plus, "New world", KeyCode::KeyN, Tone::Plain);
    let back = action(Icon::Back, "Back", "Esc", Tone::Muted);
    (
        Bar::Browse,
        strip(),
        children![
            (action(Icon::Play, "Play", "Enter", Tone::Accent), Act::Play),
            (
                action(Icon::Pencil, "Rename", KeyCode::KeyR, Tone::Plain),
                Act::Rename
            ),
            (
                action(Icon::Copy, "Duplicate", KeyCode::KeyC, Tone::Plain),
                Act::Duplicate
            ),
            (
                action(Icon::Trash, "Delete", "Del", Tone::Danger),
                Act::Delete
            ),
            (
                action(Icon::Sort, "Sort", KeyCode::KeyT, Tone::Plain),
                Act::Sort
            ),
            (new_world, Nav::To(Menu::NewGame)),
            (back, Nav::Back, Cancels),
        ],
    )
}

/// Applying or dropping the typed name.
fn rename() -> impl Bundle {
    (
        Bar::Rename,
        strip(),
        children![
            (
                action(Icon::Check, "Save name", "Enter", Tone::Accent),
                Act::Confirm
            ),
            (
                action(Icon::Close, "Cancel", "Esc", Tone::Plain),
                Act::Cancel
            ),
        ],
    )
}

/// The two answers to the delete question. Keeping is the one selected first.
fn delete() -> impl Bundle {
    let remove = action(Icon::Trash, "Delete for good", "Right, Enter", Tone::Danger);
    (
        Bar::Delete,
        strip(),
        children![
            (
                action(Icon::Close, "Keep", "Esc", Tone::Plain),
                Act::Cancel,
                Choice(false),
                Selected(true),
            ),
            (remove, Act::Confirm, Choice(true), Selected(false)),
        ],
    )
}

/// The three bars, of which the one that fits the mode is drawn.
pub(super) fn bars() -> impl Bundle {
    (
        column(percent(100), 0.0),
        children![browse(), rename(), delete()],
    )
}
