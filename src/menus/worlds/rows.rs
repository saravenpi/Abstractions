//! Filling the worlds table: the texts of the drawn rows and the lines around them.

use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use super::bars::{Bar, Choice};
use super::view::{Empty, Hint, Invite, Listing, Summary};
use super::when::relative;
use super::{Listed, Mode, Notice, VISIBLE_ROWS, Worlds};
use crate::menus::name::with_caret;
use crate::saves::{now, play_time};
use crate::ui::{Selected, Tone};

/// A row button of the pool, counted from the top of the table.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Row(pub usize);

/// Which text of a row a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The name of the world.
    Name,
    /// The planet and the system the player is on.
    Place,
    /// The seed.
    Seed,
    /// The time played.
    Played,
    /// When the world was last played.
    Last,
}

/// A text of the row in this slot.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Cell(pub usize, pub Part);

/// Returns the text and the tone of one cell of `world`. The selected row shows the name being
/// typed, or the delete question, in place of its name and place.
pub(super) fn cell_text(
    world: &Listed,
    part: Part,
    mode: Option<&Mode>,
    stamp: u64,
) -> (String, Tone) {
    let save = &world.entry.save;
    match (part, mode) {
        (Part::Name, Some(Mode::Rename(typed))) => (with_caret(typed), Tone::Accent),
        (Part::Place, Some(Mode::Rename(_))) => ("Type the new name".to_string(), Tone::Muted),
        (Part::Name, Some(Mode::Delete { .. })) => (format!("Delete {}?", save.name), Tone::Danger),
        (Part::Place, Some(Mode::Delete { .. })) => {
            ("It cannot be brought back".to_string(), Tone::Muted)
        }
        (Part::Name, _) => (save.name.clone(), Tone::Plain),
        (Part::Place, _) => (world.place.clone(), Tone::Muted),
        (Part::Seed, _) => (save.seed.to_string(), Tone::Muted),
        (Part::Played, _) => (play_time(save.play_seconds), Tone::Muted),
        (Part::Last, _) => (relative(stamp, save.last_played), Tone::Muted),
    }
}

/// Returns the line under the table: how many worlds there are, their order and which are drawn.
pub(super) fn summary(worlds: &Worlds) -> String {
    let (first, count) = (worlds.window.top + 1, worlds.list.len());
    let last = (worlds.window.top + VISIBLE_ROWS).min(count);
    let order = worlds.sort.label();
    match count {
        0 => String::new(),
        1 => "1 world".to_string(),
        n if n <= VISIBLE_ROWS => format!("{n} worlds, by {order}"),
        n => format!("{n} worlds, by {order}, {first} to {last} shown"),
    }
}

/// What the table is drawn from.
type Source<'w> = (Res<'w, Worlds>, Res<'w, Mode>);

/// Writes the texts of the drawn rows, hides the rows past the end of the list and marks the
/// selected row. Does nothing while neither the list nor the mode changed.
pub(super) fn show(
    (worlds, mode): Source,
    fresh: Query<(), Added<Row>>,
    mut cells: Query<(&Cell, &mut Text, &mut Tone)>,
    mut rows: Query<(&Row, &mut Node, &mut Selected)>,
) {
    if !(worlds.is_changed() || mode.is_changed() || !fresh.is_empty()) {
        return;
    }
    let (stamp, chosen) = (now(), worlds.window.cursor - worlds.window.top);
    for (Cell(slot, part), mut text, mut tone) in &mut cells {
        let Some(world) = worlds.at(*slot) else {
            continue;
        };
        let editing = (*slot == chosen).then_some(&*mode);
        let (shown, wanted) = cell_text(world, *part, editing, stamp);
        text.set_if_neq(Text(shown));
        tone.set_if_neq(wanted);
    }
    for (row, mut node, mut selected) in &mut rows {
        let display = match worlds.at(row.0) {
            Some(_) => Display::Flex,
            None => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
        selected.set_if_neq(Selected(row.0 == chosen && !worlds.list.is_empty()));
    }
}

/// The summary, the notice and the two choices of the delete question.
type Lines<'w, 's> = (
    Query<'w, 's, &'static mut Text, (With<Summary>, Without<Hint>)>,
    Query<'w, 's, &'static mut Text, (With<Hint>, Without<Summary>)>,
    Query<'w, 's, (&'static Choice, &'static mut Selected)>,
);

/// Writes the summary and the notice, and marks the choice Enter applies to a delete.
pub(super) fn show_lines(
    (worlds, mode): Source,
    notice: Res<Notice>,
    (mut summaries, mut hints, mut choices): Lines,
) {
    if !(worlds.is_changed() || mode.is_changed() || notice.is_changed()) {
        return;
    }
    for mut text in &mut summaries {
        text.set_if_neq(Text(summary(&worlds)));
    }
    for mut text in &mut hints {
        text.set_if_neq(Text(notice.0.clone()));
    }
    for (choice, mut selected) in &mut choices {
        let armed = *mode == Mode::Delete { armed: choice.0 };
        selected.set_if_neq(Selected(armed));
    }
}

/// Filter of the two blocks of which one shows: the table of worlds or the invitation.
type Block = Or<(With<Empty>, With<Listing>)>;

/// Filter of everything that is neither of those two blocks.
type NotBlock = (Without<Empty>, Without<Listing>);

/// Returns the display of something drawn only when `shown`.
fn display(shown: bool) -> Display {
    if shown { Display::Flex } else { Display::None }
}

/// Shows the bar of actions that fits the mode, and the invitation to create a world, which
/// takes the focus, when there is none. The table and its column names show only when there
/// is a world to list.
pub(super) fn show_bars(
    (worlds, mode): Source,
    mut bars: Query<(&Bar, &mut Node), NotBlock>,
    mut blocks: Query<(&mut Node, Has<Empty>), Block>,
    invites: Query<Entity, With<Invite>>,
    mut focus: ResMut<InputFocus>,
) {
    if !(worlds.is_changed() || mode.is_changed()) {
        return;
    }
    let empty = worlds.list.is_empty();
    for (bar, mut node) in &mut bars {
        let wanted = display(bar.shown(&mode, empty));
        if node.display != wanted {
            node.display = wanted;
        }
    }
    for (mut node, invitation) in &mut blocks {
        let wanted = display(invitation == empty);
        if node.display != wanted {
            node.display = wanted;
        }
    }
    let invite = invites.iter().next().filter(|_| empty);
    if let Some(entity) = invite.filter(|entity| focus.get() != Some(*entity)) {
        focus.set(entity, FocusCause::Navigated);
    }
}
