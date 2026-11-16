//! Row widgets of the settings menu.

use bevy::prelude::*;

use super::actions::SettingsAction;
use super::field::{Bind, Field};
use crate::ui::{button, button_with, label, row, theme};

/// Shows the live value of one setting.
#[derive(Component)]
pub struct Value(pub Field);

/// Fixed-width label that keeps the columns aligned.
fn name(text: &str) -> impl Bundle {
    (
        label(text),
        Node {
            width: px(theme::LABEL_WIDTH),
            ..default()
        },
    )
}

/// Row with minus and plus buttons around a live value.
pub fn stepper(text: &str, field: Field) -> impl Bundle {
    let value = (
        label(""),
        Value(field),
        Node {
            width: px(theme::VALUE_WIDTH),
            justify_content: JustifyContent::Center,
            ..default()
        },
    );
    (
        row(),
        children![
            name(text),
            (
                button("-", theme::STEP_WIDTH),
                SettingsAction::Adjust(field, -1)
            ),
            value,
            (
                button("+", theme::STEP_WIDTH),
                SettingsAction::Adjust(field, 1)
            ),
        ],
    )
}

/// Row with a single button that shows the value and flips it.
pub fn toggle(text: &str, field: Field) -> impl Bundle {
    let shown = button_with(
        theme::VALUE_WIDTH + 2.0 * theme::STEP_WIDTH,
        (label(""), Value(field)),
    );
    (
        row(),
        children![name(text), (shown, SettingsAction::Adjust(field, 1))],
    )
}

/// Row with a button that shows a key and waits for a new one when clicked.
pub fn binding(text: &str, bind: Bind) -> impl Bundle {
    let shown = button_with(
        theme::VALUE_WIDTH + 2.0 * theme::STEP_WIDTH,
        (label(""), Value(Field::Key(bind))),
    );
    (
        row(),
        children![name(text), (shown, SettingsAction::Rebind(bind))],
    )
}
