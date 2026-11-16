//! Row widgets of the settings menu.

use bevy::prelude::*;

use super::actions::SettingsAction;
use super::field::{Bind, Field};
use crate::ui::{Tone, label, row_with, step_button, theme};

/// Shows the live value of one setting.
#[derive(Component)]
pub struct Value(pub Field);

/// Muted setting name on the left of a row.
fn name(text: &str) -> impl Bundle {
    label(text, theme::FONT_BODY, Tone::Muted)
}

/// Live value text.
fn value(field: Field) -> impl Bundle {
    (label("", theme::FONT_BODY, Tone::Plain), Value(field))
}

/// Row with a value between minus and plus buttons.
pub fn stepper(text: &str, field: Field) -> impl Bundle {
    let cluster = (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::SPACE / 2.0),
            ..default()
        },
        children![
            (step_button("<"), SettingsAction::Adjust(field, -1)),
            (
                Node {
                    width: px(theme::SPACE * 9.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![value(field)]
            ),
            (step_button(">"), SettingsAction::Adjust(field, 1)),
        ],
    );
    (
        row_with(1, name(text), cluster),
        SettingsAction::Adjust(field, 1),
    )
}

/// Row whose value flips when activated.
pub fn toggle(text: &str, field: Field) -> impl Bundle {
    (
        row_with(1, name(text), value(field)),
        SettingsAction::Adjust(field, 1),
    )
}

/// Row that shows a key and waits for a new one when activated.
pub fn binding(text: &str, bind: Bind) -> impl Bundle {
    (
        row_with(1, name(text), value(Field::Key(bind))),
        SettingsAction::Rebind(bind),
    )
}
