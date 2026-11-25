//! 24-slot 6x4 pack grid widget with 3D item previews, stack counts, gun ammo, and wear bars.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::bench::Bench;
use super::click::{ArmorPress, Press};
use super::glyph::wear_bar;
use super::models::ItemPreview;
use super::render3d::preview_badge;
use crate::condition::{PARTS, Part};
use crate::craft::armor::ArmorSlot;
use crate::craft::build::Place;
use crate::craft::pack::{BAR_SLOTS, GRID_SLOTS, Pack, SlotEntry};
use crate::ui::{Icon, empty_state, label, text, theme};

/// Width and height in pixels of one square grid cell (`56 + 10 + 6 * 56 + 5 * 6 = 432 <= 448`).
pub const CELL: f32 = 56.0;

/// Grid columns of the 24-slot pack panel.
const COLS: usize = 6;

/// Electric cyan accent used for shielded Aegis armor badges.
const CYAN: Color = Color::srgb(0.220, 0.851, 1.0);

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
        SlotEntry::Armor(id) => {
            let armor = pack.armor(id)?;
            let color = if armor.shielded { CYAN } else { theme::ACCENT };
            Some((
                format!("{}%", (armor.defense() * 100.0).round() as u32),
                color,
            ))
        }
    }
}

/// Spawns one equipped armor slot cell (`HEAD`, `CHEST`, `LEGS`, `BOOTS`) that unequips on click.
fn armor_cell(pack: &Pack, slot: ArmorSlot) -> impl Bundle {
    let worn = pack.equipped_armor(slot);
    let preview = worn.map(|a| ItemPreview::Armor {
        slot: a.slot,
        mineral: a.mineral,
        shielded: a.shielded,
    });
    let badge = worn.map(|a| {
        let color = if a.shielded { CYAN } else { theme::ACCENT };
        (format!("{}%", (a.defense() * 100.0).round() as u32), color)
    });
    let fill = if worn.is_some() {
        theme::SLOT_FULL
    } else {
        theme::SLOT_EMPTY
    };
    let tag = slot.label();
    let spawn = SpawnWith(move |cell: &mut ChildSpawner| {
        if let Some(p) = preview {
            cell.spawn(preview_badge(p, CELL - 16.0, false));
        } else {
            cell.spawn((text(tag, 11.0, theme::TEXT_2), TextLayout::no_wrap()));
        }
        if let Some((words, color)) = badge {
            cell.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(3),
                    bottom: px(2),
                    ..default()
                },
                text(words, 11.0, color),
                TextLayout::no_wrap(),
            ));
        }
    });
    (
        Button,
        Hovered::default(),
        ArmorPress(slot),
        Node {
            width: px(CELL),
            height: px(CELL),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(fill),
        Children::spawn(spawn),
    )
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

/// Renders the 4-slot vertical Armor Column beside the 6x4 pack storage grid (`grid[6..30]`).
pub fn pack_grid(parent: &mut ChildSpawnerCommands, pack: &Pack, bench: &Bench, place: &Place) {
    let row_node = Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::FlexStart,
        column_gap: px(10.0),
        ..default()
    };
    let armor_col = Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(6.0),
        ..default()
    };
    let grid_node = Node {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::px(COLS as u16, CELL),
        column_gap: px(6.0),
        row_gap: px(6.0),
        ..default()
    };
    parent.spawn(row_node).with_children(|row| {
        row.spawn(armor_col).with_children(|col| {
            for slot in ArmorSlot::ALL {
                col.spawn(armor_cell(pack, slot));
            }
        });
        row.spawn(grid_node).with_children(|grid| {
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
    });
    let avg_def = Part::ALL
        .into_iter()
        .map(|p| pack.defense_for(p))
        .sum::<f32>()
        / PARTS as f32;
    let shield_max = pack.total_shield_max();
    parent.spawn((
        Node {
            margin: UiRect::top(px(theme::S2)),
            ..default()
        },
        text(
            format!(
                "DEF {}% · SHD {} J",
                (avg_def * 100.0).round() as u32,
                shield_max.round() as u32
            ),
            theme::FONT_SMALL,
            if shield_max > 0.0 {
                CYAN
            } else {
                theme::TEXT_2
            },
        ),
    ));
    if pack.grid[BAR_SLOTS..].iter().all(Option::is_none)
        && pack.items.is_empty()
        && pack.guns.is_empty()
        && pack.armors.is_empty()
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
            margin: UiRect::top(px(theme::S1)),
            ..default()
        },
        label(
            "Click swap/equip  Right-click split  Shift-click move to bar",
            theme::FONT_SMALL,
            crate::ui::Tone::Muted,
        ),
    ));
}
