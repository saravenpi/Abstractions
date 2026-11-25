//! Right-hand Crafting Workshop recipe inspector: 3D preview, Have/Need checklist, hints, and batch craft buttons.

use bevy::prelude::*;

use super::bench::Bench;
use super::catalog_ui::{filtered_recipes, output_preview};
use super::click::Press;
use super::models::ItemPreview;
use super::render3d::preview_badge;
use crate::craft::build::catalog::{Blueprint, IngredientNeed, missing_hint};
use crate::craft::build::catalog_craft::{ingredient_have, ingredient_sample, max_craftable};
use crate::craft::matter::Mineral;
use crate::craft::pack::Pack;
use crate::ui::{Icon, Tone, action, frame, label, notice_row, text, theme};

/// Spawns one Have/Need ingredient requirement row with its 3D thumbnail and gathering hint.
fn ingredient_row(
    parent: &mut ChildSpawnerCommands,
    need: IngredientNeed,
    req_count: u16,
    pack: &Pack,
    chosen: Mineral,
) {
    let have = ingredient_have(pack, &need, chosen);
    let ok = have >= u32::from(req_count);
    let (item_label, sample_stuff) = ingredient_sample(&need, pack, chosen);
    let status_color = if ok { theme::ACCENT } else { theme::DANGER };
    let count_line = format!("{have} / {req_count}");
    let row_node = Node {
        width: percent(100),
        padding: UiRect::all(px(theme::S2)),
        column_gap: px(theme::S2),
        align_items: AlignItems::Center,
        ..default()
    };
    let right_col = Node {
        flex_grow: 1.0,
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let preview = ItemPreview::from_stuff(&sample_stuff);
    parent
        .spawn((row_node, BackgroundColor(theme::SLOT_EMPTY)))
        .with_children(|r| {
            r.spawn(preview_badge(preview, 28.0, false));
            r.spawn(right_col).with_children(|c| {
                c.spawn((
                    Node {
                        width: percent(100),
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: px(theme::S2),
                        ..default()
                    },
                    children![
                        label(item_label, theme::FONT_SMALL, Tone::Plain),
                        text(count_line, theme::FONT_SMALL, status_color),
                    ],
                ));
                if !ok {
                    c.spawn((
                        text(missing_hint(&need, chosen), theme::FONT_SMALL, theme::WARN),
                        TextLayout::default(),
                    ));
                }
            });
        });
}

/// Renders the recipe header, optional mineral cycler, and full Have/Need checklist on the right.
pub fn recipe_inspector(parent: &mut ChildSpawnerCommands, pack: &Pack, bench: &Bench) {
    let recipes = filtered_recipes(bench.category, pack, bench.bullet_mineral);
    let Some(bp) = recipes.get(bench.recipe_idx.min(recipes.len().saturating_sub(1))) else {
        return;
    };
    let chosen = bench.bullet_mineral;
    let inspector_node = Node {
        flex_grow: 1.0,
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::S2),
        padding: UiRect::all(px(theme::S3)),
        ..default()
    };
    parent
        .spawn((inspector_node, BackgroundColor(theme::PANEL)))
        .with_children(|col| {
            spawn_inspector_body(col, bp, pack, chosen);
        });
}

/// Populates the inspector contents inside `col`.
fn spawn_inspector_body(
    col: &mut ChildSpawnerCommands,
    bp: &Blueprint,
    pack: &Pack,
    chosen: Mineral,
) {
    let preview = output_preview(bp, chosen);
    col.spawn((
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::S3),
            margin: UiRect::bottom(px(theme::S2)),
            ..default()
        },
        children![
            preview_badge(preview, 56.0, false),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    min_width: px(0),
                    ..default()
                },
                children![
                    label(bp.name, theme::FONT_BODY, Tone::Plain),
                    label(bp.summary, theme::FONT_SMALL, Tone::Muted),
                ]
            )
        ],
    ));
    if bp
        .inputs
        .iter()
        .any(|i| i.need == IngredientNeed::ChosenMineral)
    {
        let ore_name = crate::craft::pack::called(&crate::craft::matter::Stuff::Mineral(chosen));
        col.spawn((
            Press::CycleMineral,
            frame(),
            Node {
                padding: UiRect::new(
                    px(theme::INSET),
                    px(theme::INSET),
                    px(theme::S1),
                    px(theme::S1),
                ),
                align_items: AlignItems::Center,
                column_gap: px(theme::S2),
                align_self: AlignSelf::FlexStart,
                ..default()
            },
            children![
                preview_badge(ItemPreview::Mineral(chosen), 24.0, false),
                label(
                    format!("Ore: {ore_name} (Click / O to cycle)"),
                    theme::FONT_SMALL,
                    Tone::Plain
                ),
            ],
        ));
    }
    col.spawn(label("Requirements", theme::FONT_SMALL, Tone::Muted));
    for ing in bp.inputs {
        ingredient_row(col, ing.need, ing.count, pack, chosen);
    }
}

/// Spawns `Craft 1`, `Craft x5`, and `Craft Max` buttons + live note in `Holder::Deeds`.
pub fn catalog_deeds(parent: &mut ChildSpawnerCommands, pack: &Pack, bench: &Bench) {
    let recipes = filtered_recipes(bench.category, pack, bench.bullet_mineral);
    if let Some(bp) = recipes.get(bench.recipe_idx.min(recipes.len().saturating_sub(1))) {
        let max_n = max_craftable(bp, pack, bench.bullet_mineral);
        parent.spawn((
            Press::CraftBatch(1),
            action(
                Some(Icon::Plus),
                "Craft 1",
                "Enter",
                if max_n >= 1 {
                    Tone::Accent
                } else {
                    Tone::Muted
                },
            ),
        ));
        parent.spawn((
            Press::CraftBatch(5),
            action(
                None,
                "Craft x5",
                "C",
                if max_n >= 5 { Tone::Plain } else { Tone::Muted },
            ),
        ));
        parent.spawn((
            Press::CraftBatch(0),
            action(
                None,
                &format!("Craft Max ({max_n})"),
                "M",
                if max_n >= 1 { Tone::Plain } else { Tone::Muted },
            ),
        ));
    }
    if !bench.note.is_empty() {
        parent.spawn(notice_row(
            Icon::Info,
            (
                text(&bench.note, theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap(),
            ),
        ));
    }
}
