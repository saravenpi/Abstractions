//! Texts that name keys: they follow the keyboard layout of the player and the key bindings.

use bevy::prelude::*;

use super::item::label;
use super::key_line::KeyLine;
use super::style::Tone;
use super::widgets::text;
use crate::settings::{Keymap, Labels, Persistent, Settings};

/// A text that names keys, in the color `color`.
pub fn key_text(line: impl Into<KeyLine>, size: f32, color: Color) -> impl Bundle {
    let line = line.into();
    (text(line.plain(), size, color), line)
}

/// A one-line text that names keys, in the tone `tone`.
pub fn key_label(line: impl Into<KeyLine>, size: f32, tone: Tone) -> impl Bundle {
    let line = line.into();
    (label(line.plain(), size, tone), line)
}

/// Run condition: a label, a binding or a line changed.
pub(super) fn stale(
    labels: Labels,
    settings: Option<Res<Persistent<Settings>>>,
    lines: Query<(), Changed<KeyLine>>,
) -> bool {
    let rebound = settings.is_some_and(|settings| settings.is_changed());
    labels.is_changed() || rebound || !lines.is_empty()
}

/// Writes every text that names keys.
pub(super) fn write(
    labels: Labels,
    settings: Option<Res<Persistent<Settings>>>,
    mut lines: Query<(&KeyLine, &mut Text)>,
) {
    let map = settings.map_or_else(Keymap::default, |settings| settings.controls.keys);
    let labels = labels.get();
    for (line, mut text) in &mut lines {
        text.set_if_neq(Text(line.words(&map, &labels)));
    }
}
