//! 24-slot 6x4 pack grid widget with 3D item previews, stack counts, gun ammo, and wear bars.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::bench::Bench;
use super::click::Press;
use super::glyph::wear_bar;
use super::models::ItemPreview;
use super::render3d::preview_badge;
use crate::craft::build::Place;
use crate::craft::pack::{BAR_SLOTS, GRID_SLOTS, Pack, SlotEntry};
use crate::ui::{Icon, empty_state, label, text, theme};

/// Width and height in pixels of one square grid cell (`6 * 64 + 5 * 8 = 424 <= 448`).
pub const CELL: f32 = 64.0;

/// Grid columns of the 24-slot pack panel.
const COLS: usize = 6;

/// Returns badge text `(label, color)` drawn in the bottom-right corner of an occupied cell.
fn badge_overlay(pack: &Pack, entry: &SlotEntry) -> Option<(String, Color)> {
    match *entry {
        SlotEntry::Stack { count, .. } => Some((format!("{count}"), theme::TEXT)),
        SlotEntry::Gun(id) => {
            let gun = pack.gun(id)?;
            let cap = gun.kind.stats().capacity;
            let color = if gun.loaded == 0 {
                theme::WARN
            } else {
                theme::ACCENT
            };
            Some((format!("{}/{cap}", gun.loaded), color))
        }
        SlotEntry::Tool(_) => None,
    }
}

/// Spawns one square inventory cell (`size x size`) for `entry` with 3D thumbnail and overlays.
pub fn slot_cell(
    pack: &Pack,
    entry: Option<&SlotEntry>,
    place: &Place,
    (size, selected, picked): (f32, bool, bool),
    press: Press,
) -> impl Bundle {
    let preview = entry.and_then(|e| ItemPreview::from_slot(pack, e));
    let overlay = entry.and_then(|e| badge_overlay(pack, e));
    let tool_wear = entry.and_then(|e| match *e {
        SlotEntry::Tool(id) => pack.item(id).map(|it| it.worn(place).0),
        _ => None,
    });
    let fill = if picked {
        Color::srgba(1.0, 0.86, 0.35, 0.22)
    } else if selected {
        theme::SELECT
    } else if entry.is_some() {
        theme::SLOT_FULL
    } else {
        theme::SLOT_EMPTY
    };
    let bar_color = if picked {
        theme::PIN
    } else if selected {
        theme::ACCENT
    } else {
        Color::NONE
    };
    let thumb = size - 16.0;
    let spawn = SpawnWith(move |cell: &mut ChildSpawner| {
        if selected || picked {
            cell.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    bottom: px(0),
                    width: px(3),
                    ..default()
                },
                BackgroundColor(bar_color),
            ));
        }
        if let Some(p) = preview {
            cell.spawn(preview_badge(p, thumb, false));
        }
        if let Some((words, color)) = overlay {
            cell.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(4),
                    bottom: px(2),
                    ..default()
                },
                text(words, theme::FONT_SMALL, color),
                TextLayout::no_wrap(),
            ));
        }
        if let Some(worn) = tool_wear {
            cell.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(3),
                    ..default()
                },
                wear_bar((size - 12.0).max(20.0), worn),
            ));
        }
    });
    (
        Button,
        Hovered::default(),
        press,
        Node {
            width: px(size),
            height: px(size),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(fill),
        Children::spawn(spawn),
    )
}

/// Renders the 6x4 pack storage grid (`grid[6..30]`) and usage hint.
pub fn pack_grid(parent: &mut ChildSpawnerCommands, pack: &Pack, bench: &Bench, place: &Place) {
    let grid_node = Node {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::px(COLS as u16, CELL),
        column_gap: px(theme::S2),
        row_gap: px(theme::S2),
        ..default()
    };
    parent.spawn(grid_node).with_children(|grid| {
        for offset in 0..GRID_SLOTS {
            let slot_idx = BAR_SLOTS + offset;
            let selected = bench.selected_slot == Some(slot_idx);
            let picked = bench.picked_slot == Some(slot_idx);
            grid.spawn(slot_cell(
                pack,
                pack.grid[slot_idx].as_ref(),
                place,
                (CELL, selected, picked),
                Press::GridSlot(slot_idx),
            ));
        }
    });
    if pack.grid[BAR_SLOTS..].iter().all(Option::is_none)
        && pack.items.is_empty()
        && pack.guns.is_empty()
    {
        parent.spawn(empty_state(
            Icon::Pack,
            "Pack slots empty",
            "Break rocks, chop trees, or dig ground with H to gather minerals and wood.",
        ));
        return;
    }
    parent.spawn((
        Node {
            margin: UiRect::top(px(theme::S2)),
            ..default()
        },
        label(
            "Click swap/stack  Right-click split  Shift-click move to bar",
            theme::FONT_SMALL,
            crate::ui::Tone::Muted,
        ),
    ));
}
