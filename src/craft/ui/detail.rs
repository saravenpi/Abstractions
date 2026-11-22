//! Focused pack item details: what the selected tool or material stack is, its mass and share of
//! the pack load, its condition and joint life when it is a tool, and four property bars.

use bevy::prelude::*;

use super::glyph::{CARD, badge};
use super::lines::{Line, What};
use super::scene::{Ground, Shape};
use super::sheet::{Gauge, Sheet, blows, gauges, share, sheet};
use super::tune::{FULL_HARDNESS, FULL_STIFFNESS, FULL_STRENGTH, FULL_TOUGHNESS};
use crate::craft::build::Place;
use crate::craft::matter::Material;
use crate::craft::pack::{Item, Pack, Stack};
use crate::ui::units::fixed;
use crate::ui::{Icon, STAT_METER, meter, text, theme};

/// Width in pixels of the label column of an inspector bar, wide enough for "Toughness".
pub const BAR_LABEL: f32 = 96.0;

/// Zeroed sheet for a broken tool whose recipe lost a material.
const ZERO_SHEET: Sheet = Sheet {
    mass: 0.0,
    reach: 0.0,
    speed: 0.0,
    depth: 0.0,
    life: 0.0,
};

/// Details of the focused row of the pack, shown beside the list.
#[derive(Debug, Clone, PartialEq)]
pub struct Detail {
    /// Its full name.
    pub name: String,
    /// What it is made of, or its grade and treatments.
    pub signature: String,
    /// Its mass in kilograms.
    pub mass: f32,
    /// Share of the carried load, 0 to 1.
    pub load_share: f32,
    /// Its silhouette, when it is a tool.
    pub shape: Shape,
    /// Its material icon, when it is a stack.
    pub icon: Option<Icon>,
    /// Remaining condition, 0 broken to 1 new, when it is a tool.
    pub condition: Option<f32>,
    /// Blows the weakest joint lasts, when it is a tool.
    pub life: Option<f32>,
    /// Hotbar slot number, 1 to 6, when it is a slotted tool.
    pub slot: Option<usize>,
    /// Four property bars for the focused entry.
    pub gauges: [Gauge; 4],
}

/// Returns the four property bars of `material`: hardness, toughness, strength and stiffness.
fn material_gauges(material: &Material) -> [Gauge; 4] {
    let g = |label, val, full, unit, dec| Gauge {
        label,
        share: share(val, full),
        value: format!("{} {unit}", fixed(val, dec)),
    };
    [
        g("Hardness", material.hardness, FULL_HARDNESS, "Mohs", 1),
        g("Toughness", material.toughness, FULL_TOUGHNESS, "MPa", 2),
        g("Strength", material.strength, FULL_STRENGTH, "MPa", 0),
        g("Stiffness", material.stiffness, FULL_STIFFNESS, "GPa", 1),
    ]
}

/// Returns the details of tool `item` from its row `line` at `place`, with `load_share`.
fn tool_detail(item: &Item, line: &Line, place: &Place, load_share: f32) -> Detail {
    let numbers = item
        .build()
        .map_or(ZERO_SHEET, |b| sheet(&b, item.intent(), place));
    Detail {
        name: line.name.clone(),
        signature: line.signature.clone(),
        mass: line.mass,
        load_share,
        shape: line.shape,
        icon: line.icon,
        condition: line.wear.map(|w| (1.0 - w).clamp(0.0, 1.0)),
        life: Some(blows(numbers.life)),
        slot: line.slot,
        gauges: gauges(&numbers, item.intent()),
    }
}

/// Returns the details of material `stack` from its row `line`, with `load_share`.
fn stack_detail(line: &Line, stack: &Stack, load_share: f32) -> Detail {
    Detail {
        name: line.name.clone(),
        signature: line.signature.clone(),
        mass: line.mass,
        load_share,
        shape: line.shape,
        icon: line.icon,
        condition: None,
        life: None,
        slot: None,
        gauges: material_gauges(&stack.material),
    }
}

/// Returns the details of the pack entry at `cursor` in `all`, or nothing when empty or out of
/// range.
pub fn detail(pack: &Pack, all: &[Line], cursor: usize, ground: &Ground) -> Option<Detail> {
    let line = all.get(cursor)?;
    let total = ground.burden.load.max(pack.mass());
    let share = if total > f32::EPSILON {
        (line.mass / total).clamp(0.0, 1.0)
    } else {
        0.0
    };
    match line.what {
        What::Item(id) => pack
            .item(id)
            .map(|item| tool_detail(item, line, &ground.place, share)),
        What::Stack(at) => pack
            .stacks
            .get(at)
            .map(|stack| stack_detail(line, stack, share)),
    }
}

/// Returns the header of the inspector: icon or tool picture beside full name and signature.
fn header(shown: &Detail) -> impl Bundle {
    let row = Node {
        width: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(theme::S3),
        ..default()
    };
    let words = Node {
        flex_grow: 1.0,
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (
        row,
        children![
            badge(shown.shape, shown.icon, CARD, false),
            (
                words,
                children![
                    text(&shown.name, theme::FONT_BODY, theme::TEXT),
                    text(&shown.signature, theme::FONT_SMALL, theme::TEXT_2),
                ]
            ),
        ],
    )
}

/// Spawns the mass, load share and optional tool condition lines under `parent`.
fn summary_lines(parent: &mut ChildSpawnerCommands, shown: &Detail) {
    let pct = fixed(shown.load_share * 100.0, 0);
    let mass = format!("{} kg, {pct} % of load", fixed(shown.mass, 2));
    parent.spawn((
        text(mass, theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
    ));
    let Some(cond) = shown.condition else {
        return;
    };
    let color = match cond {
        c if c < 0.25 => theme::DANGER,
        c if c < 0.50 => theme::WARN,
        _ => theme::TEXT_2,
    };
    let life = fixed(shown.life.unwrap_or(0.0), 0);
    let words = match shown.slot {
        Some(slot) => format!(
            "Condition {} %, {life} blows, #{slot}",
            fixed(cond * 100.0, 0)
        ),
        None => format!("Condition {} %, {life} blows", fixed(cond * 100.0, 0)),
    };
    parent.spawn((text(words, theme::FONT_SMALL, color), TextLayout::no_wrap()));
}

/// Returns one property bar row for `gauge`: label, meter and value on one small line.
fn gauge_row(gauge: &Gauge) -> impl Bundle {
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        min_height: px(theme::LINE_SMALL),
        flex_shrink: 0.0,
        ..default()
    };
    let label_box = Node {
        width: px(BAR_LABEL),
        flex_shrink: 0.0,
        ..default()
    };
    (
        row,
        children![
            (
                text(gauge.label, theme::FONT_SMALL, theme::TEXT_2),
                TextLayout::no_wrap(),
                label_box,
            ),
            meter(STAT_METER, gauge.share),
            (
                text(&gauge.value, theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap(),
            ),
        ],
    )
}

/// Spawns the detail inspector of `shown` under `parent`, or nothing when `shown` is empty.
pub fn panel(parent: &mut ChildSpawnerCommands, shown: Option<&Detail>) {
    let Some(shown) = shown else {
        return;
    };
    let card = Node {
        width: percent(100),
        max_height: percent(100),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(theme::S4)),
        row_gap: px(theme::S2),
        overflow: Overflow::clip_y(),
        ..default()
    };
    let column = || Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    parent
        .spawn((card, BackgroundColor(theme::PANEL_BLUR)))
        .with_children(|box_node| {
            box_node.spawn(header(shown));
            box_node
                .spawn(column())
                .with_children(|col| summary_lines(col, shown));
            box_node.spawn(column()).with_children(|bars| {
                for gauge in &shown.gauges {
                    bars.spawn(gauge_row(gauge));
                }
            });
        });
}
