//! Typing on the map screen: a click on the search box or the slash key starts a search,
//! letters then narrow the results, Backspace edits the search and Escape clears it. A pin
//! being renamed takes the keys instead.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::Computer;
use super::board::Board;
use super::sift::type_into;
use super::view::{SEARCH_HINT, SearchBox, SearchText};
use crate::ui::Tone;

/// The key that starts a search.
pub(super) const SEARCH_KEY: KeyCode = KeyCode::Slash;

/// Where the letters typed on the map screen go. While they go anywhere, the keys of the map
/// (the map key, the engage key, the pin key, the reset key) type instead.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub enum Typing {
    /// Nowhere: every key does what it does on the map.
    #[default]
    Off,
    /// Into the search.
    Search,
    /// Into the name of the pin at this index of the list.
    Name(usize, String),
}

impl Typing {
    /// Returns true while the letters go nowhere.
    pub fn is_off(&self) -> bool {
        *self == Typing::Off
    }
}

/// Applies one pressed key and the `text` it types to the search, and returns true when the
/// search changed. Escape clears it and ends the search. The text is what the layout types,
/// dead keys and AltGr included, so an accent typed in two presses arrives whole.
fn edit(filter: &mut String, (key, text): (&Key, &str), typing: &mut Typing) -> bool {
    match key {
        Key::Escape => {
            *typing = Typing::Off;
            !std::mem::take(filter).is_empty()
        }
        Key::Backspace => filter.pop().is_some(),
        _ => type_into(filter, text),
    }
}

/// Starts a search on its key, edits it with the keys pressed this frame and ends it on Escape,
/// clearing what was typed. Escape also clears a search left over once typing went elsewhere
/// and ended. The list is then built again from its first row. Keys pressed before the screen
/// opened are dropped on the frame it opens.
pub(super) fn type_filter(
    mut typed: MessageReader<KeyboardInput>,
    (keys, opened): (Res<ButtonInput<KeyCode>>, Res<State<Computer>>),
    mut typing: ResMut<Typing>,
    mut board: ResMut<Board>,
) {
    if opened.is_changed() {
        typed.clear();
        return;
    }
    if typing.is_off() && keys.just_pressed(SEARCH_KEY) {
        *typing = Typing::Search;
        typed.clear();
        return;
    }
    let mut filter = board.filter.clone();
    let mut changed = false;
    for key in typed.read().filter(|key| key.state == ButtonState::Pressed) {
        match (&*typing, &key.logical_key) {
            (Typing::Search, _) | (Typing::Off, Key::Escape) => {
                let typed = key.text.as_deref().unwrap_or_default();
                changed |= edit(&mut filter, (&key.logical_key, typed), &mut typing);
            }
            _ => {}
        }
    }
    if !changed {
        return;
    }
    (board.filter, board.stale) = (filter, true);
    (board.cursor, board.top) = (0, 0);
}

/// Starts a search when the search box is clicked.
pub(super) fn on_search(
    event: On<Activate>,
    boxes: Query<(), With<SearchBox>>,
    mut typing: ResMut<Typing>,
) {
    if boxes.contains(event.entity) && typing.is_off() {
        *typing = Typing::Search;
    }
}

/// Writes the search box: the search with a caret while it is typed, what was typed after it
/// ended, or the hint while nothing is.
pub(super) fn show_search(
    typing: Res<Typing>,
    board: Res<Board>,
    mut texts: Query<(&mut Text, &mut Tone), With<SearchText>>,
) {
    if !(typing.is_changed() || board.is_changed()) {
        return;
    }
    let (words, tone) = match (&*typing, board.filter.is_empty()) {
        (Typing::Search, _) => (format!("{}_", board.filter), Tone::Plain),
        (_, true) => (SEARCH_HINT.to_string(), Tone::Muted),
        (_, false) => (board.filter.clone(), Tone::Plain),
    };
    for (mut text, mut shown) in &mut texts {
        text.set_if_neq(Text(words.clone()));
        shown.set_if_neq(tone);
    }
}

/// Ends any typing and clears the search, so neither outlives the screen: the keys of the
/// game work again once it closes, and Escape closes it once it opens again.
pub(super) fn stop_typing(mut typing: ResMut<Typing>, mut board: ResMut<Board>) {
    *typing = Typing::Off;
    if !board.filter.is_empty() {
        (board.filter, board.stale) = (String::new(), true);
    }
}
