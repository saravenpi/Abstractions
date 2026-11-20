//! Writes the texts of the open guide from its shelf: the count line, the drawn rows of the list
//! and the card of the selected species.

use bevy::prelude::*;

use super::card::{Card, Row, card};
use super::open::Shelf;
use super::seen::Seen;
use super::view::{Called, Facts, Kind, Slot, Subtitle, Tally, Title};
use crate::ui::{Selected, Tone, caps, text, theme};
use crate::world::Terrain;

/// Returns `words` cut to `most` characters, ending in two dots when cut.
pub(crate) fn clip(words: &str, most: usize) -> String {
    if words.chars().count() <= most {
        return words.to_string();
    }
    let kept: String = words.chars().take(most.saturating_sub(2)).collect();
    format!("{kept}..")
}

/// The rows of the list and the texts inside them.
type Rows<'w, 's> = (
    Query<'w, 's, (&'static Slot, &'static mut Selected, &'static Children)>,
    Query<
        'w,
        's,
        (&'static mut Text, &'static mut Tone, Has<Called>),
        Or<(With<Called>, With<Kind>)>,
    >,
);

/// Writes the drawn rows of the list: the name of each species, or unknown and dimmed, its kind,
/// and which one is selected.
pub(super) fn list(shelf: Res<Shelf>, (mut slots, mut texts): Rows) {
    let entries = &shelf.book.entries;
    for (slot, mut selected, children) in &mut slots {
        let index = shelf.window.index(entries.len(), slot.0);
        let entry = index.and_then(|i| entries.get(i));
        selected.set_if_neq(Selected(
            index == Some(shelf.window.cursor) && entry.is_some(),
        ));
        for child in children {
            let Ok((mut words, mut tone, name)) = texts.get_mut(*child) else {
                continue;
            };
            let (wanted, look) = match (entry, name) {
                (Some(e), true) => (
                    e.name.as_str(),
                    [Tone::Muted, Tone::Plain][usize::from(e.seen)],
                ),
                (Some(e), false) => (e.kind, Tone::Muted),
                (None, _) => ("", Tone::Muted),
            };
            words.0.clear();
            words.0.push_str(wanted);
            tone.set_if_neq(look);
        }
    }
}

/// Spawns one fact under `parent`: its muted label on the left and its value on the right, cut
/// to `most` characters across.
fn fact(parent: &mut ChildSpawnerCommands, (name, value): (&str, &str), most: usize) {
    let small = |words: String, color: Color| {
        (text(words, theme::FONT_SMALL, color), TextLayout::no_wrap())
    };
    let line = Node {
        width: percent(100),
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(theme::GAP_S),
        ..default()
    };
    let value = clip(value, most.saturating_sub(name.chars().count() + 1));
    parent.spawn((
        line,
        children![
            small(name.to_string(), theme::TEXT_2),
            small(value, theme::TEXT)
        ],
    ));
}

/// Spawns the rows of `card` under `parent`, a heading after the first keeping room over it.
pub(crate) fn rows(parent: &mut ChildSpawnerCommands, card: &Card, most: usize) {
    for (at, row) in card.rows.iter().enumerate() {
        match row {
            Row::Heading(name) => {
                let gap = if at == 0 { 0.0 } else { theme::GAP_S };
                parent.spawn((
                    caps(name),
                    Node {
                        margin: UiRect::top(px(gap)),
                        ..default()
                    },
                ));
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
    (&'static mut Text, Has<Title>, Has<Subtitle>),
    Or<(With<Title>, With<Subtitle>, With<Tally>)>,
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
    for (mut words, title, subtitle) in &mut texts {
        words.0 = match (title, subtitle) {
            (true, _) => clip(&shown.title, shelf.chars * 7 / 9),
            (_, true) => clip(&shown.subtitle, shelf.chars),
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
