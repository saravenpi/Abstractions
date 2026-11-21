//! Row widgets of the settings menu. Every row has the same three parts on the same lines: the
//! icon of the setting, its name, and its value at the left of the value zone.

use bevy::prelude::*;

use super::actions::SettingsAction;
use super::field::{Bind, Field};
use super::icons;
use crate::ui::{Icon, Tone, entry, label, row_with, step_button, theme, value_cell};

/// Shows the live value of one setting.
#[derive(Component)]
pub struct Value(pub Field);

/// The icon of a setting and its muted name, on the left of a row.
fn name(field: Field, text: &str) -> impl Bundle {
    entry(icons::of(field), text, theme::FONT_BODY, Tone::Muted)
}

/// Live value text.
pub fn value(field: Field, size: f32) -> impl Bundle {
    (label("", size, Tone::Plain), Value(field))
}

/// The room of the two buttons that step a value.
fn steps() -> Node {
    Node {
        width: px(theme::VALUE_CLUSTER - theme::VALUE_WIDTH - theme::S1),
        flex_shrink: 0.0,
        column_gap: px(theme::S1),
        ..default()
    }
}

/// The value zone of a row: the live value at its left, then `tail`, the step buttons or
/// their empty room.
fn zone(field: Field, tail: impl Bundle) -> impl Bundle {
    let node = Node {
        width: px(theme::VALUE_CLUSTER),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::S1),
        ..default()
    };
    let shown = value_cell(JustifyContent::FlexStart, value(field, theme::FONT_BODY));
    (node, children![shown, tail])
}

/// Row with a value and the two buttons that step it down and up.
pub fn stepper(text: &str, field: Field) -> impl Bundle {
    let less = (
        step_button(Icon::ChevronLeft),
        SettingsAction::Adjust(field, -1),
    );
    let more = (
        step_button(Icon::ChevronRight),
        SettingsAction::Adjust(field, 1),
    );
    let buttons = (steps(), children![less, more]);
    (
        row_with(1, name(field, text), zone(field, buttons)),
        SettingsAction::Adjust(field, 1),
    )
}

/// Row whose value flips when activated.
pub fn toggle(text: &str, field: Field) -> impl Bundle {
    (
        row_with(1, name(field, text), zone(field, steps())),
        SettingsAction::Adjust(field, 1),
    )
}

/// Row that shows the key of an action and waits for a new one when activated.
pub fn binding(bind: Bind) -> impl Bundle {
    let field = Field::Key(bind);
    (
        row_with(1, name(field, bind.label()), zone(field, steps())),
        SettingsAction::Rebind(bind),
    )
}
