//! The pack pane drawn: one group per kind, each a section with its icon and its rows, or the
//! empty state that says how to fill the pack.
//!
//! A row is two lines: the name, then the mass with the grade or what the focused tool card takes
//! from it, and the wear bar of a tool.

use bevy::prelude::*;

use super::click::Press;
use super::kind::Kind;
use super::reason::missing;
use super::scene::Row;
use crate::craft::build::Intent;
use crate::fauna::guide::show::clip;
use crate::ui::units::fixed;
use crate::ui::{
    Icon, Reveal, Tone, empty_state, groups, label, line, meter, row_tall, rows, section, theme,
};

/// Characters of body text a name shows: the width of the pack pane less the icon slot.
pub const NAME: usize = 20;

/// Characters of small text the line under a name shows, beside the wear bar.
pub const FACT: usize = 20;

/// Width in pixels of the wear bar of a tool.
pub const WEAR: f32 = 32.0;

/// Returns the line under the name of `row`: its mass, then what the focused card takes from it,
/// or its grade and treatments.
pub fn fact(row: &Row) -> String {
    let mass = fixed(row.line.mass, 2);
    match row.uses {
        Some(kg) => format!("uses {} of {mass} kg", fixed(kg, 2)),
        None => format!("{mass} kg, {}", row.line.signature),
    }
}

/// Returns the row `one` of the pack, at `at`.
fn row(at: usize, one: &Row) -> impl Bundle {
    let grow = || Node {
        flex_grow: 1.0,
        min_width: px(0),
        ..default()
    };
    let name = label(clip(&one.line.name, NAME), theme::FONT_BODY, Tone::Plain);
    let under = label(clip(&fact(one), FACT), theme::FONT_SMALL, Tone::Muted);
    let worn = match one.line.wear {
        Some(_) => Visibility::Inherited,
        None => Visibility::Hidden,
    };
    let bar = (meter(WEAR, 1.0 - one.line.wear.unwrap_or(0.0)), worn);
    (
        Press::Row(at),
        row_tall(
            one.line.icon,
            one.lit,
            (line(), children![(name, grow())]),
            (line(), children![(under, grow()), bar]),
        ),
    )
}

/// Returns the group of `kind`: its title and its rows, the lit one kept in view when `reveal`.
fn group(kind: Kind, members: Vec<(usize, Row)>, reveal: bool) -> impl Bundle {
    let spawn = SpawnWith(move |parent: &mut ChildSpawner| {
        for (at, one) in &members {
            let mut made = parent.spawn(row(*at, one));
            if reveal && one.lit {
                made.insert(Reveal);
            }
        }
    });
    section(kind.icon(), kind.name(), (rows(), Children::spawn(spawn)))
}

/// Spawns the groups of `all` under `parent`, or the empty state of an empty pack. The cursor row
/// is kept in view when `reveal`.
pub fn list(parent: &mut ChildSpawnerCommands, all: &[Row], reveal: bool) {
    if all.is_empty() {
        let why = missing(Intent::Cut, &[]);
        parent.spawn(empty_state(Icon::Pack, "The pack is empty", &why));
        return;
    }
    let mut column = parent.spawn(groups());
    column.with_children(|inner| {
        for kind in Kind::ALL {
            let members: Vec<(usize, Row)> = all
                .iter()
                .enumerate()
                .filter(|(_, one)| one.line.kind == kind)
                .map(|(at, one)| (at, one.clone()))
                .collect();
            if !members.is_empty() {
                inner.spawn(group(kind, members, reveal));
            }
        }
    });
}
