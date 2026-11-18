//! The main button at the bottom of the panel: the next thing to do with the course, said in
//! words with its key, and the small button beside it that clears the course.

use bevy::prelude::*;

use super::board::Board;
use super::course::{primary, words};
use super::layout::BUTTON;
use super::route::{Destination, RouteStatus, Target};
use super::view::Act;
use crate::menus::key_name;
use crate::settings::{Persistent, Settings};
use crate::ui::{Icon, Tone, frame, icon, label, theme};

/// Marks the main button. Its [`Act`] says what it does now.
#[derive(Component)]
pub(super) struct Primary;

/// Marks the small button beside the main one that clears the course.
#[derive(Component)]
pub(super) struct Spare;

/// Which text of the main button a text is.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Face {
    /// The icon of the action.
    Glyph,
    /// What the button does.
    Words,
    /// The key that does the same.
    Key,
}

/// The main button: a framed strip as wide as the room beside the small button, with the icon
/// and the words on the left and the key on the right. The words are cut before the key is.
pub(super) fn button() -> impl Bundle {
    let node = Node {
        flex_grow: 1.0,
        min_width: px(0),
        height: px(BUTTON),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(theme::GAP_S),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        border: UiRect::all(px(theme::LINE)),
        overflow: Overflow::clip(),
        ..default()
    };
    let left = Node {
        min_width: px(0),
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        overflow: Overflow::clip(),
        ..default()
    };
    let glyph = (icon(Icon::MapPin, Tone::Accent), Face::Glyph);
    let said = (label("", theme::FONT_BODY, Tone::Accent), Face::Words);
    let key = (label("", theme::FONT_SMALL, Tone::Muted), Face::Key);
    (
        (frame(), Primary, Act::Set, node),
        BorderColor::all(theme::TEXT),
        children![(left, children![glyph, said]), key],
    )
}

/// Returns the icon of an action of the footer.
fn glyph(act: Act) -> Icon {
    match act {
        Act::Set => Icon::MapPin,
        Act::Engage => Icon::Send,
        Act::Clear => Icon::Eraser,
    }
}

/// Returns the name of the key that does `act`, with `engage` the engage key of the settings.
fn key(act: Act, engage: KeyCode) -> String {
    match act {
        Act::Set => "Enter".to_string(),
        Act::Engage => key_name(engage),
        Act::Clear => "Del".to_string(),
    }
}

/// Returns whether the main button, or the small one when `main` is false, is shown for the
/// next action `next` and the course `course`.
fn shown(main: bool, next: Option<Act>, course: Option<Target>) -> bool {
    match main {
        true => next.is_some(),
        false => course.is_some() && next != Some(Act::Clear),
    }
}

/// What the footer reads to say what its buttons do.
type Course<'w> = (
    Res<'w, Board>,
    Res<'w, Destination>,
    Res<'w, RouteStatus>,
    Res<'w, Persistent<Settings>>,
);

/// The two buttons of the footer: the main one and the small one.
type Buttons<'w, 's> = Query<
    'w,
    's,
    (&'static mut Act, &'static mut Node, Has<Primary>),
    Or<(With<Primary>, With<Spare>)>,
>;

/// Gives the main button the action that comes next and its words and key, shows the small
/// button while there is a course it does not already clear, and hides what has nothing to do.
/// Does nothing while none of what it reads changed.
pub(super) fn offer(
    (board, course, status, settings): Course,
    fresh: Query<(), Added<Primary>>,
    mut buttons: Buttons,
    mut faces: Query<(&Face, &mut Text)>,
) {
    let changed = board.is_changed() || course.is_changed() || status.is_changed();
    if !(changed || settings.is_changed() || !fresh.is_empty()) {
        return;
    }
    let row = board.lines.get(board.cursor);
    let next = primary(row.map(|line| line.target), course.0, *status);
    for (mut act, mut node, main) in &mut buttons {
        let display = [Display::None, Display::Flex][usize::from(shown(main, next, course.0))];
        if node.display != display {
            node.display = display;
        }
        if let Some(wanted) = next.filter(|_| main) {
            act.set_if_neq(wanted);
        }
    }
    let Some(act) = next else {
        return;
    };
    let key = key(act, settings.controls.keys.engage);
    let name = row.map_or("", |line| line.title.as_str());
    write(&mut faces, act, name, &key);
}

/// Writes the icon, the words and the key of the main button for `act`, with `name` the name
/// of the selected row and `key` the name of the key that does it.
fn write(faces: &mut Query<(&Face, &mut Text)>, act: Act, name: &str, key: &str) {
    for (face, mut text) in faces {
        let wanted = match face {
            Face::Glyph => glyph(act).glyph(),
            Face::Words => words(act, name),
            Face::Key => key.to_string(),
        };
        text.set_if_neq(Text(wanted));
    }
}
