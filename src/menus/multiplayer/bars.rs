//! The bars of actions of the multiplayer menu: one while browsing and one while a form is
//! filled, each action with its icon, its label and its key.

use bevy::prelude::*;

use super::Pane;
use super::act::Act;
use crate::menus::Nav;
use crate::ui::{Cancels, Icon, Tone, action, column, strip};

/// The actions on the list and going back.
fn browse() -> impl Bundle {
    let direct = action(Icon::Link, "Direct connect", "D", Tone::Plain);
    let back = action(Icon::Back, "Back", "Esc", Tone::Muted);
    (
        Pane::List,
        strip(),
        children![
            (action(Icon::Play, "Join", "Enter", Tone::Accent), Act::Join),
            (direct, Act::Direct),
            (action(Icon::Plus, "Add", "A", Tone::Plain), Act::Add),
            (action(Icon::Pencil, "Edit", "E", Tone::Plain), Act::Edit),
            (
                action(Icon::Trash, "Remove", "Del", Tone::Danger),
                Act::Remove
            ),
            (
                action(Icon::Reload, "Refresh", "F5", Tone::Plain),
                Act::Refresh
            ),
            (back, Nav::Back, Cancels),
        ],
    )
}

/// Applying or dropping the form.
fn form() -> impl Bundle {
    (
        Pane::Form,
        strip(),
        children![
            (
                action(Icon::Check, "Confirm", "Enter", Tone::Accent),
                Act::Confirm
            ),
            (
                action(Icon::Close, "Cancel", "Esc", Tone::Plain),
                Act::Cancel
            ),
        ],
    )
}

/// The two bars, of which the one that fits the mode is drawn.
pub(super) fn bars() -> impl Bundle {
    (column(percent(100), 0.0), children![browse(), form()])
}
