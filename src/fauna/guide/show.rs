//! Writes the texts of the open guide from its shelf: the count line, the drawn rows of the list
//! and the card of the selected species, every row of both with its icon.

use bevy::prelude::*;

use super::book::Entry;
use super::card::{Card, Row, card};
use super::icons::{of_label, sign};
use super::open::Shelf;
use super::seen::Seen;
use super::view::{Called, Facts, Kind, Mark, Sign, Slot, Subtitle, Tally, Title};
use crate::ui::{Selected, Tone, caps, icon_sized, text, theme};
use crate::world::Terrain;

/// Returns `words` cut to `most` characters, ending in two dots when cut.
pub(crate) fn clip(words: &str, most: usize) -> String {
    if words.chars().count() <= most {
        return words.to_string();
    }
    let kept: String = words.chars().take(most.saturating_sub(2)).collect();
    format!("{kept}..")
}

/// Characters of small text the icon of a row takes, with the gap after it.
pub(crate) const ICON_CHARS: usize = 3;

/// The rows of the list and the texts inside them.
type Rows<'w, 's> = (
    Query<'w, 's, (&'static Slot, &'static mut Selected, &'static Children)>,
    Query<
        'w,
        's,
        (&'static mut Text, &'static mut Tone, Has<Called>, Has<Mark>),
        Or<(With<Called>, With<Kind>, With<Mark>)>,
    >,
);

/// Returns what one text of a row says and its tone: the silhouette, the name or the kind of
/// `entry`. A species not seen yet is dimmed and wears a question mark.
fn cell(entry: Option<&Entry>, (name, mark): (bool, bool)) -> (String, Tone) {
    let Some(entry) = entry else {
        return (String::new(), Tone::Muted);
    };
    let seen = [Tone::Muted, Tone::Plain][usize::from(entry.seen)];
    match (name, mark) {
        (_, true) => (entry.icon.glyph(), seen),
        (true, _) => (entry.name.clone(), seen),
        _ => (entry.kind.to_string(), Tone::Muted),
    }
}

/// Writes the drawn rows of the list: the silhouette and the name of each species, or a question
/// mark and unknown, dimmed, its kind, and which one is selected.
pub(super) fn list(shelf: Res<Shelf>, (mut slots, mut texts): Rows) {
    let entries = &shelf.book.entries;
    for (slot, mut selected, children) in &mut slots {
        let index = shelf.window.index(entries.len(), slot.0);
        let entry = index.and_then(|i| entries.get(i));
        selected.set_if_neq(Selected(
            index == Some(shelf.window.cursor) && entry.is_some(),
        ));
        for child in children {
            let Ok((mut words, mut tone, name, mark)) = texts.get_mut(*child) else {
                continue;
            };
            let (wanted, look) = cell(entry, (name, mark));
            words.set_if_neq(Text(wanted));
            tone.set_if_neq(look);
        }
    }
}

/// Spawns the room of the icon of a row under `row`, with the icon of `label` when the guide
/// knows one. A label without an icon keeps the room, so the labels stay in line.
fn mark(row: &mut ChildSpawnerCommands, label: &str) {
    let node = Node {
        width: px(theme::ICON_SMALL),
        height: px(theme::ICON_SMALL),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    let mut slot = row.spawn(node);
    if let Some(symbol) = of_label(label) {
        slot.with_child(icon_sized(symbol, Tone::Muted, theme::ICON_SMALL));
    }
}

/// The node of one row of a card: its icon, then its words, in line.
fn line(gap: f32) -> Node {
    Node {
        width: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        margin: UiRect::top(px(gap)),
        ..default()
    }
}

/// Spawns one fact under `parent`: its icon, its muted label on the left and its value at the
/// right end, cut to `most` characters across.
fn fact(parent: &mut ChildSpawnerCommands, (name, value): (&str, &str), most: usize) {
    let small = |words: String, color: Color| {
        (text(words, theme::FONT_SMALL, color), TextLayout::no_wrap())
    };
    let grow = Node {
        flex_grow: 1.0,
        ..default()
    };
    let room = most.saturating_sub(name.chars().count() + 1 + ICON_CHARS);
    let value = clip(value, room);
    parent.spawn(line(0.0)).with_children(|row| {
        mark(row, name);
        row.spawn((grow, children![small(name.to_string(), theme::TEXT_2)]));
        row.spawn(small(value, theme::TEXT));
    });
}

/// Spawns the rows of `card` under `parent`: a heading with its icon, keeping room over it after
/// the first, a fact with its icon, or a sentence.
pub(crate) fn rows(parent: &mut ChildSpawnerCommands, card: &Card, most: usize) {
    for (at, row) in card.rows.iter().enumerate() {
        match row {
            Row::Heading(name) => {
                let gap = if at == 0 { 0.0 } else { theme::GAP_S };
                parent.spawn(line(gap)).with_children(|row| {
                    mark(row, name);
                    row.spawn(caps(name));
                });
            }
            Row::Fact(name, value) => fact(parent, (name, value), most),
            Row::Note(words) => {
                parent.spawn(text(words.clone(), theme::FONT_SMALL, theme::TEXT_2));
            }
        }
    }
}

/// The texts of the card and the count line.
type Texts<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, Has<Title>, Has<Subtitle>, Has<Sign>),
    Or<(With<Title>, With<Subtitle>, With<Tally>, With<Sign>)>,
>;

/// Writes the count line and the card of the selected species.
pub(super) fn card_of(
    (shelf, seen, terrain): (Res<Shelf>, Res<Seen>, Option<Res<Terrain>>),
    mut texts: Texts,
    facts: Query<Entity, With<Facts>>,
    mut commands: Commands,
) {
    let life = terrain.as_deref().and_then(|t| t.globe().life());
    let index = shelf.book.entries.get(shelf.window.cursor).map(|e| e.index);
    let shown = card(life, index, &seen);
    let tally = match shelf.planet.is_empty() {
        true => shelf.book.tally(),
        false => format!("{} - {}", shelf.planet, shelf.book.tally()),
    };
    let mark = sign(life, index, &seen).glyph();
    for (mut words, title, subtitle, sign) in &mut texts {
        words.0 = match (title, subtitle, sign) {
            (true, ..) => clip(
                &shown.title,
                (shelf.chars * 7 / 9).saturating_sub(ICON_CHARS),
            ),
            (_, true, _) => clip(&shown.subtitle, shelf.chars),
            (.., true) => mark.clone(),
            _ => tally.clone(),
        };
    }
    for column in &facts {
        commands.entity(column).despawn_children();
        commands
            .entity(column)
            .with_children(|parent| rows(parent, &shown, shelf.chars));
    }
}
