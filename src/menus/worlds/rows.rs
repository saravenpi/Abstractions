//! The world rows: building, scrolling and showing their text.

use bevy::ecs::system::SystemParam;
use bevy::input_focus::{AutoFocus, InputFocus};
use bevy::prelude::*;

use super::view::RowsBox;
use super::{EXTRA_SLOT, Pending, Row, Scroll, Selected, VISIBLE_ROWS, WorldList};
use crate::menus::slots::Slot;
use crate::saves::{WorldEntry, ago, now, play_time};
use crate::ui::{Tone, item_with, label, text, theme};

/// Wraps one row so the whole row can be hidden when it scrolls out.
#[derive(Component)]
pub(super) struct Wrap(usize);

/// Which line of a row a text is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    Title,
    Detail,
}

/// A text of the row at this index.
#[derive(Component)]
pub(super) struct RowText(usize, Part);

/// What the rebuild needs to replace the rows.
#[derive(SystemParam)]
pub(super) struct Build<'w, 's> {
    commands: Commands<'w, 's>,
    boxes: Query<'w, 's, Entity, With<RowsBox>>,
    extras: Query<'w, 's, (Entity, &'static Slot)>,
    focus: ResMut<'w, InputFocus>,
}

/// Returns the muted detail line of a world.
fn detail(entry: &WorldEntry, now: u64) -> String {
    let save = &entry.save;
    format!(
        "seed {} - {} - {} played",
        save.seed,
        ago(now, save.last_played),
        play_time(save.play_seconds)
    )
}

/// The title and detail texts of a row.
fn lines(index: usize) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                label("", theme::FONT_BODY, Tone::Plain),
                RowText(index, Part::Title)
            ),
            (
                label("", theme::FONT_SMALL, Tone::Muted),
                RowText(index, Part::Detail)
            ),
        ],
    )
}

/// One row: a wrapper around a button with the name and the detail line.
fn row(index: usize, select: bool) -> impl Bundle {
    let button = (item_with(0, lines(index)), Row(index), Slot(index));
    (
        Wrap(index),
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            let mut entity = parent.spawn(button);
            if select {
                entity.insert(AutoFocus);
            }
        })),
    )
}

/// Replaces the rows when the list changes, focusing the selected one.
pub(super) fn rebuild(mut build: Build, list: Res<WorldList>, selected: Res<Selected>) {
    let Ok(rows_box) = build.boxes.single() else {
        return;
    };
    build
        .commands
        .entity(rows_box)
        .despawn_related::<Children>();
    let wanted = selected.0.min(list.0.len().saturating_sub(1));
    for index in 0..list.0.len() {
        build
            .commands
            .spawn((row(index, index == wanted), ChildOf(rows_box)));
    }
    if list.0.is_empty() {
        build.commands.spawn((
            text("No worlds yet", theme::FONT_BODY, theme::MUTED),
            ChildOf(rows_box),
        ));
        let new_world = build.extras.iter().find(|(_, slot)| slot.0 == EXTRA_SLOT);
        if let Some((entity, _)) = new_world {
            build
                .focus
                .set(entity, bevy::input_focus::FocusCause::Navigated);
        }
    }
}

/// Keeps the focused row inside the visible window and hides the others.
pub(super) fn scroll(
    focus: Res<InputFocus>,
    list: Res<WorldList>,
    rows: Query<&Row>,
    mut offset: ResMut<Scroll>,
    mut wraps: Query<(&Wrap, &mut Node)>,
) {
    let mut start = offset.0;
    if let Some(Row(index)) = focus.get().and_then(|entity| rows.get(entity).ok()) {
        start = start
            .min(*index)
            .max((index + 1).saturating_sub(VISIBLE_ROWS));
    }
    start = start.min(list.0.len().saturating_sub(VISIBLE_ROWS));
    offset.set_if_neq(Scroll(start));
    for (wrap, mut node) in &mut wraps {
        let shown = (start..start + VISIBLE_ROWS).contains(&wrap.0);
        let display = if shown { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}

/// Writes each row's text, turning the pending row into its delete confirmation.
pub(super) fn show_rows(
    list: Res<WorldList>,
    pending: Res<Pending>,
    mut texts: Query<(&RowText, &mut Text, &mut Tone)>,
) {
    let stamp = now();
    for (RowText(index, part), mut text, mut tone) in &mut texts {
        let Some(entry) = list.0.get(*index) else {
            continue;
        };
        let asking = pending.0 == Some(*index);
        let (shown, wanted) = match (part, asking) {
            (Part::Title, true) => (format!("Delete {}?", entry.save.name), Tone::Accent),
            (Part::Title, false) => (entry.save.name.clone(), Tone::Plain),
            (Part::Detail, true) => ("Enter confirm, Esc cancel".to_string(), Tone::Muted),
            (Part::Detail, false) => (detail(entry, stamp), Tone::Muted),
        };
        text.set_if_neq(Text(shown));
        tone.set_if_neq(wanted);
    }
}
