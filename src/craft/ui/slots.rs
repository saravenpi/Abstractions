//! The hotbar under both Pack and Workshop panes: shows all 6 hotbar slots with 3D previews.

use bevy::prelude::*;

use super::bench::Bench;
use super::click::Press;
use super::glyph::{ROW, wear_bar};
use super::models::ItemPreview;
use super::render3d::preview_badge;
use crate::craft::build::Place;
use crate::craft::pack::{Pack, SLOTS, SlotEntry, called};
use crate::craft::wield::KEYS;
use crate::fauna::guide::show::clip;
use crate::ui::{Selected, Tone, frame, key_cap, label, line, text, theme};

/// Characters of small text the word for a tool or item in a slot shows.
pub const NAME: usize = 7;

/// Width in pixels of the wear bar of a slot.
pub const WEAR: f32 = 32.0;

/// Returns the node of a slot: a sixth of the row, its key and its contents on one line.
fn node() -> Node {
    Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        min_width: px(0),
        height: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        padding: UiRect::horizontal(px(theme::INSET)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Spawns one live hotbar slot for `pack.grid[at]`, lit when held or picked.
fn slot_bundle(at: usize, pack: &Pack, place: &Place, held: bool, picked: bool) -> impl Bundle {
    let entry = pack.grid[at];
    let preview = entry.as_ref().and_then(|e| ItemPreview::from_slot(pack, e));
    let (title, sub, wear, warn_sub) = match entry {
        Some(SlotEntry::Tool(id)) => match pack.item(id) {
            Some(it) => (clip(&it.name(), NAME), None, Some(it.worn(place).0), false),
            None => ("Empty".to_string(), None, None, false),
        },
        Some(SlotEntry::Gun(id)) => match pack.gun(id) {
            Some(g) => {
                let cap = g.kind.stats().capacity;
                (
                    clip(&g.name(), NAME),
                    Some(format!("{}/{cap}", g.loaded)),
                    None,
                    g.loaded == 0,
                )
            }
            None => ("Empty".to_string(), None, None, false),
        },
        Some(SlotEntry::Armor(id)) => match pack.armor(id) {
            Some(a) => (clip(&a.name(), NAME), None, Some(a.wear), false),
            None => ("Empty".to_string(), None, None, false),
        },
        Some(SlotEntry::Stack { stuff, count, .. }) => (
            clip(called(&stuff), NAME),
            Some(format!("x{count}")),
            None,
            false,
        ),
        None => ("Empty".to_string(), None, None, false),
    };
    let tone = if entry.is_some() {
        Tone::Plain
    } else {
        Tone::Muted
    };
    let spawn = SpawnWith(move |parent: &mut ChildSpawner| {
        parent.spawn(key_cap(KEYS[at]));
        if let Some(p) = preview {
            parent.spawn(preview_badge(p, ROW, false));
        } else {
            parent.spawn(Node {
                width: px(ROW),
                height: px(ROW),
                flex_shrink: 0.0,
                ..default()
            });
        }
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                min_width: px(0),
                ..default()
            })
            .with_children(|col| {
                col.spawn(label(title, theme::FONT_SMALL, tone));
                if let Some(worn) = wear {
                    col.spawn((line(), children![wear_bar(WEAR, worn)]));
                } else if let Some(s) = sub {
                    col.spawn(text(
                        s,
                        theme::FONT_SMALL,
                        if warn_sub { theme::WARN } else { theme::ACCENT },
                    ));
                }
            });
    });
    (
        Press::Slot(at),
        frame(),
        Selected(held || picked),
        node(),
        Children::spawn(spawn),
    )
}

/// Spawns all six hotbar slots under `parent` reading directly from `pack.grid[0..6]`.
pub fn slots_from_pack(
    parent: &mut ChildSpawnerCommands,
    pack: &Pack,
    bench: &Bench,
    place: &Place,
) {
    for at in 0..SLOTS {
        let held = at == pack.bar.held;
        let picked = bench.picked_slot == Some(at) || bench.selected_slot == Some(at);
        parent.spawn(slot_bundle(at, pack, place, held, picked));
    }
}
