//! Categorized Crafting Workshop category strip and affordability-filtered recipe cards.

use bevy::prelude::*;

use super::bench::Bench;
use super::click::Press;
use super::models::ItemPreview;
use super::render3d::preview_badge;
use crate::craft::build::catalog::{Blueprint, BlueprintOutput, Category, catalog};
use crate::craft::build::catalog_craft::max_craftable;
use crate::craft::matter::Mineral;
use crate::craft::pack::Pack;
use crate::ui::{Icon, Selected, Tone, empty_state, frame, label, text, theme};

/// Returns blueprints belonging to `category`, checked against `pack` with `chosen` mineral.
pub fn filtered_recipes(
    category: Category,
    pack: &Pack,
    chosen: Mineral,
) -> Vec<&'static Blueprint> {
    catalog()
        .iter()
        .filter(|bp| match category {
            Category::All => true,
            Category::Craftable => max_craftable(bp, pack, chosen) > 0,
            Category::Tools => false,
            other => bp.category == other,
        })
        .collect()
}

/// Resolves the canonical 3D preview of `bp`'s produced item.
pub fn output_preview(bp: &Blueprint, chosen: Mineral) -> ItemPreview {
    match bp.output {
        BlueprintOutput::Stack { stuff, .. } => ItemPreview::from_stuff(&stuff),
        BlueprintOutput::ChosenBullet(_) => ItemPreview::Bullet(chosen),
        BlueprintOutput::Firearm(kind) => ItemPreview::Gun(kind, chosen),
        BlueprintOutput::Armor { slot, shielded } => ItemPreview::Armor {
            slot,
            mineral: chosen,
            shielded,
        },
    }
}

/// Spawns the six category tabs (`All`, `Craftable`, `Basics`, `Weapons`, `Base`, `Tools`).
pub fn categories(parent: &mut ChildSpawnerCommands, active: Category) {
    let bar = Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(theme::S2),
        row_gap: px(theme::S1),
        margin: UiRect::bottom(px(theme::S3)),
        ..default()
    };
    parent.spawn(bar).with_children(|row| {
        for cat in Category::ALL {
            row.spawn((
                Press::Category(cat),
                frame(),
                Selected(cat == active),
                Node {
                    height: px(theme::ROW_HEIGHT),
                    padding: UiRect::horizontal(px(theme::INSET)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![label(
                    cat.label(),
                    theme::FONT_SMALL,
                    if cat == active {
                        Tone::Plain
                    } else {
                        Tone::Muted
                    }
                )],
            ));
        }
    });
}

/// Spawns one recipe row card in the left list of the Crafting Workshop.
fn recipe_row(
    list_idx: usize,
    bp: &Blueprint,
    (pack, chosen, selected): (&Pack, Mineral, bool),
) -> impl Bundle {
    let max_n = max_craftable(bp, pack, chosen);
    let badge_text = if max_n > 0 {
        format!("Ready ({max_n})")
    } else {
        "Missing".to_string()
    };
    let badge_color = if max_n > 0 {
        theme::ACCENT
    } else {
        theme::WARN
    };
    let yield_str = match bp.output {
        BlueprintOutput::Stack { count, .. } | BlueprintOutput::ChosenBullet(count)
            if count > 1 =>
        {
            format!("{} x{count}", bp.name)
        }
        _ => bp.name.to_string(),
    };
    let preview = output_preview(bp, chosen);
    (
        Press::Recipe(list_idx),
        frame(),
        Selected(selected),
        Node {
            width: percent(100),
            padding: UiRect::all(px(theme::S2)),
            column_gap: px(theme::S2),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            preview_badge(preview, 36.0, false),
            (
                Node {
                    flex_grow: 1.0,
                    min_width: px(0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    label(yield_str, theme::FONT_BODY, Tone::Plain),
                    text(badge_text, theme::FONT_SMALL, badge_color),
                ]
            )
        ],
    )
}

/// Spawns the scrollable recipe selector list on the left side of the Crafting Workshop.
pub fn recipe_list(parent: &mut ChildSpawnerCommands, pack: &Pack, bench: &Bench) {
    let recipes = filtered_recipes(bench.category, pack, bench.bullet_mineral);
    if recipes.is_empty() {
        parent.spawn(empty_state(
            Icon::Pack,
            "Nothing craftable yet",
            "Break rocks, chop trees, or dig terrain with H, or switch to All to view recipes.",
        ));
        return;
    }
    let active = bench.recipe_idx.min(recipes.len() - 1);
    let col = Node {
        width: px(theme::span(5)),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::S2),
        ..default()
    };
    parent.spawn(col).with_children(|list| {
        for (i, bp) in recipes.iter().enumerate() {
            list.spawn(recipe_row(i, bp, (pack, bench.bullet_mineral, i == active)));
        }
    });
}
