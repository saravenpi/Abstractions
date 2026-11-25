//! Dual-grid Chest overlay: 6x4 chest storage above, 6x4 pack grid and 6-slot hotbar below.

use abstractions_net::chest::{CHEST_SLOTS, ITEM_STACK, PlacedProp};
use abstractions_net::wire::ChestAct;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use super::{ChestCursor, ChestScreen, ChestWorld, OpenChest, codec, ui_ops};
use crate::craft::pack::{BAR_SLOTS, GRID_SLOTS, Pack, SlotEntry};
use crate::craft::ui::models::ItemPreview;
use crate::craft::ui::render3d::preview_badge;
use crate::ui::{
    Icon, Tone, action, back, heading, label, page, page_body, page_foot, page_head, page_title,
    strip, text, theme,
};

/// Square pixel dimension of one dual-grid inventory cell.
const CELL: f32 = 58.0;

/// Grid columns shared by both the chest grid and the pack grid.
const COLS: u16 = 6;

/// Dynamic container sections rebuilt when chest or pack state changes.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChestHolder {
    /// 24-slot 6x4 chest storage grid.
    ChestGrid,
    /// 24-slot 6x4 pack grid plus 6-slot hotbar row.
    PackGrid,
}

/// Interactive buttons on the dual-grid chest overlay.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChestPress {
    /// Click on a chest or pack slot.
    Slot(ChestCursor),
    /// Deposit matching pack stacks into the open chest.
    QuickStack,
    /// Compact and sort slots inside the open chest.
    Sort,
    /// Close the chest overlay.
    Close,
}

/// Builds one Bevy 0.20 interactive inventory cell with 3D thumbnail, badge text, and highlight.
fn slot_cell(
    preview: Option<ItemPreview>,
    badge: Option<(String, Color)>,
    (selected, occupied): (bool, bool),
    press: ChestPress,
) -> impl Bundle {
    let fill = match (selected, occupied) {
        (true, _) => theme::SELECT,
        (false, true) => theme::SLOT_FULL,
        (false, false) => theme::SLOT_EMPTY,
    };
    let spawn = SpawnWith(move |cell: &mut ChildSpawner| {
        if selected {
            let bar = Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                bottom: px(0),
                width: px(3),
                ..default()
            };
            cell.spawn((bar, BackgroundColor(theme::ACCENT)));
        }
        if let Some(item) = preview {
            cell.spawn(preview_badge(item, CELL - 14.0, false));
        }
        if let Some((words, color)) = badge {
            let corner = Node {
                position_type: PositionType::Absolute,
                right: px(4),
                bottom: px(2),
                ..default()
            };
            cell.spawn((
                corner,
                text(words, theme::FONT_SMALL, color),
                TextLayout::no_wrap(),
            ));
        }
    });
    let box_node = Node {
        width: px(CELL),
        height: px(CELL),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };
    (
        Button,
        Hovered::default(),
        press,
        box_node,
        BackgroundColor(fill),
        Children::spawn(spawn),
    )
}

/// Returns badge text for a pack [`SlotEntry`] (stack count or loaded ammo).
fn pack_badge(pack: &Pack, entry: &SlotEntry) -> Option<(String, Color)> {
    match *entry {
        SlotEntry::Stack { count, .. } => Some((format!("{count}"), theme::TEXT)),
        SlotEntry::Gun(id) => {
            let gun = pack.gun(id)?;
            let color = [theme::ACCENT, theme::WARN][usize::from(gun.loaded == 0)];
            Some((
                format!("{}/{}", gun.loaded, gun.kind.stats().capacity),
                color,
            ))
        }
        _ => None,
    }
}

/// Spawns the dual-grid chest overlay skeleton on entering [`ChestScreen::Open`].
pub(super) fn spawn(mut commands: Commands) {
    let head = page_head(children![page_title("Chest")]);
    let actions = (
        strip(),
        children![
            (
                ChestPress::QuickStack,
                action(Icon::Pack, "Quick Stack", "Q", Tone::Plain)
            ),
            (
                ChestPress::Sort,
                action(Icon::Workshop, "Sort", "R", Tone::Plain)
            ),
        ],
    );
    let col = Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::S2),
        ..default()
    };
    let hint = label(
        "Click swap/stack  Right-click split  Shift-click transfer",
        theme::FONT_SMALL,
        Tone::Muted,
    );
    let body = (
        page_body(FlexDirection::Column),
        children![
            heading("Chest Storage"),
            (ChestHolder::ChestGrid, col.clone()),
            actions,
            heading("Pack & Hotbar"),
            (ChestHolder::PackGrid, col),
            hint,
        ],
    );
    let foot = page_foot(children![(ChestPress::Close, back("Close"))]);
    commands.spawn((DespawnOnExit(ChestScreen::Open), page(head, body, foot)));
}

/// Spawns the 24 chest slot cells into `grid`.
fn fill_chest(
    grid: &mut ChildSpawnerCommands,
    prop: Option<&PlacedProp>,
    selected: Option<ChestCursor>,
) {
    for idx in 0..CHEST_SLOTS {
        let item = prop.and_then(|p| p.slots.get(idx)).and_then(Option::as_ref);
        let badge = item
            .filter(|i| i.kind == ITEM_STACK)
            .map(|i| (format!("{}", i.count), theme::TEXT));
        let sel = selected == Some(ChestCursor::Chest(idx));
        grid.spawn(slot_cell(
            item.and_then(codec::preview),
            badge,
            (sel, item.is_some()),
            ChestPress::Slot(ChestCursor::Chest(idx)),
        ));
    }
}

/// Spawns pack slots in `range` into `grid`.
fn fill_pack(
    grid: &mut ChildSpawnerCommands,
    pack: &Pack,
    range: std::ops::Range<usize>,
    selected: Option<ChestCursor>,
) {
    for idx in range {
        let entry = pack.grid[idx].as_ref();
        let prev = entry.and_then(|e| ItemPreview::from_slot(pack, e));
        let badge = entry.and_then(|e| pack_badge(pack, e));
        let sel = selected == Some(ChestCursor::Pack(idx));
        grid.spawn(slot_cell(
            prev,
            badge,
            (sel, entry.is_some()),
            ChestPress::Slot(ChestCursor::Pack(idx)),
        ));
    }
}

/// Populates the 6x4 chest grid and 6x4 pack + 6-slot hotbar grids when state changes.
pub(super) fn paint(
    mut commands: Commands,
    (world, pack, open): (Res<ChestWorld>, Res<Pack>, Res<OpenChest>),
    holders: Query<(Entity, &ChestHolder)>,
) {
    let prop = open.id.and_then(|id| world.chests.find(id));
    let grid_node = Node {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::px(COLS, CELL),
        column_gap: px(theme::S2),
        row_gap: px(theme::S2),
        ..default()
    };
    for (entity, holder) in &holders {
        commands.entity(entity).despawn_related::<Children>();
        let node = grid_node.clone();
        match holder {
            ChestHolder::ChestGrid => {
                commands.entity(entity).with_children(|p| {
                    p.spawn(node)
                        .with_children(|g| fill_chest(g, prop, open.selected));
                });
            }
            ChestHolder::PackGrid => {
                commands.entity(entity).with_children(|p| {
                    p.spawn(node.clone()).with_children(|g| {
                        fill_pack(g, &pack, BAR_SLOTS..(BAR_SLOTS + GRID_SLOTS), open.selected)
                    });
                    p.spawn(node)
                        .with_children(|g| fill_pack(g, &pack, 0..BAR_SLOTS, open.selected));
                });
            }
        }
    }
}

/// Handles left-clicks on chest overlay slots and action buttons.
pub(super) fn on_activate(
    event: On<Activate>,
    presses: Query<&ChestPress>,
    keys: Res<ButtonInput<KeyCode>>,
    (mut world, mut pack, mut open): (ResMut<ChestWorld>, ResMut<Pack>, ResMut<OpenChest>),
    mut next: ResMut<NextState<ChestScreen>>,
) {
    let Ok(&press) = presses.get(event.entity) else {
        return;
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    match press {
        ChestPress::Slot(c) => {
            ui_ops::click_slot(&mut world, &mut pack, &mut open, c, shift);
        }
        ChestPress::QuickStack => {
            if let Some(id) = open.id {
                ui_ops::quick_stack_all(&mut world, &mut pack, id);
            }
        }
        ChestPress::Sort => {
            if let Some(id) = open.id {
                world.emit(ChestAct::Sort { id });
                open.selected = None;
            }
        }
        ChestPress::Close => next.set(ChestScreen::Closed),
    }
}

/// Splits a stack in half when right-clicking a hovered chest or pack slot.
pub(super) fn on_right_click(
    buttons: Res<ButtonInput<MouseButton>>,
    presses: Query<(&ChestPress, &Hovered)>,
    (mut world, mut pack, mut open): (ResMut<ChestWorld>, ResMut<Pack>, ResMut<OpenChest>),
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    for (&press, hovered) in &presses {
        if hovered.get()
            && let ChestPress::Slot(cursor) = press
        {
            ui_ops::split_slot(&mut world, &mut pack, &mut open, cursor);
            return;
        }
    }
}
