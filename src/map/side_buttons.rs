//! The buttons of the selected pin: Go with the key of Go beside it, Rename and Remove.

use bevy::prelude::*;

use super::side::{Do, PinButton, PinGoKey};
use crate::computer::{Keys, LIST_ROW};
use crate::settings::Keymap;
use crate::ui::{KEY_GAP, Tip, Tone, frame, key_cap, label, theme};

/// The node of a button of the selected pin.
fn button_node() -> Node {
    Node {
        height: px(LIST_ROW),
        align_items: AlignItems::Center,
        column_gap: px(KEY_GAP),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        ..default()
    }
}

/// One button of the selected pin, with its words and then its key.
pub(super) fn button(
    act: Do,
    (words, key): (&'static str, &'static str),
    tip: &'static str,
) -> impl Bundle {
    (
        frame(),
        PinButton(act),
        Tip(tip),
        button_node(),
        children![label(words, theme::FONT_SMALL, Tone::Plain), key_cap(key)],
    )
}

/// The Go button of the selected pin: its words and, in muted, the key of Go, as on the card of
/// a place.
pub(super) fn go_button() -> impl Bundle {
    let key = (PinGoKey, key_cap(Keys::of(&Keymap::default()).go));
    (
        frame(),
        PinButton(Do::Go),
        Tip("Fly to this pin"),
        button_node(),
        children![label("Go", theme::FONT_SMALL, Tone::Accent), key],
    )
}
