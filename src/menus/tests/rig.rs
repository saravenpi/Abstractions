//! Test rig: the narrowest width each node of a built page needs, from the texts it holds
//! measured with the interface font, added up as the flex layout of the interface lays them out.

use bevy::prelude::*;
use bevy::text::{FontSize, FontSource, LetterSpacing, LineBreak};

use crate::ui::{ICON_FONT, text_width};

/// Returns the pixels of a length, or 0 for auto, a share or a window unit.
pub fn pixels(value: Val) -> f32 {
    match value {
        Val::Px(pixels) => pixels,
        _ => 0.0,
    }
}

/// Returns the width the text of `entity` needs, or nothing when it has no text: all of it when
/// it stays on one line, its longest word when it wraps. An icon takes its size.
fn text_need(world: &World, entity: Entity) -> Option<f32> {
    let (text, font) = (world.get::<Text>(entity)?, world.get::<TextFont>(entity)?);
    let FontSize::Px(size) = font.font_size else {
        panic!("a text is not sized in pixels: {:?}", text.0);
    };
    if font.font == FontSource::Handle(ICON_FONT) {
        return Some(size);
    }
    let spacing = match world.get::<LetterSpacing>(entity) {
        Some(LetterSpacing::Px(spacing)) => *spacing,
        _ => 0.0,
    };
    let wraps = world
        .get::<TextLayout>(entity)
        .is_none_or(|layout| layout.linebreak != LineBreak::NoWrap);
    let measure = |words: &str| text_width(words, size, spacing);
    Some(match wraps {
        true => text.0.split_whitespace().map(measure).fold(0.0, f32::max),
        false => measure(text.0.as_str()),
    })
}

/// Whether `entity` takes room in the flow of its parent: it is shown and not placed on its own.
fn in_flow(world: &World, entity: Entity) -> bool {
    world.get::<Node>(entity).is_some_and(|node| {
        node.display != Display::None && node.position_type != PositionType::Absolute
    })
}

/// Returns the children of `entity` that take room in its flow.
fn flowing(world: &World, entity: Entity) -> Vec<Entity> {
    let Some(children) = world.get::<Children>(entity) else {
        return Vec::new();
    };
    children
        .into_iter()
        .copied()
        .filter(|child| in_flow(world, *child))
        .collect()
}

/// Returns the width `entity` takes: its fixed width when it has one, else what its content
/// needs, and never less than its least width.
pub fn need(world: &World, entity: Entity) -> f32 {
    let node = world.get::<Node>(entity).expect("a node");
    let wanted = match node.width {
        Val::Px(fixed) => fixed,
        _ => content(world, entity),
    };
    wanted.max(pixels(node.min_width))
}

/// Returns the width the content of `entity` needs with its padding: its text, or its children
/// side by side in a row, or the widest of them in a column or in a row that wraps.
pub fn content(world: &World, entity: Entity) -> f32 {
    let node = world.get::<Node>(entity).expect("a node");
    let pad = pixels(node.padding.left) + pixels(node.padding.right);
    if let Some(own) = text_need(world, entity) {
        return own + pad;
    }
    let outer = |child: Entity| {
        let margin = world.get::<Node>(child).expect("a node").margin;
        need(world, child) + pixels(margin.left) + pixels(margin.right)
    };
    let kids: Vec<f32> = flowing(world, entity).into_iter().map(outer).collect();
    let row = matches!(
        node.flex_direction,
        FlexDirection::Row | FlexDirection::RowReverse
    );
    let inner = match row && node.flex_wrap == FlexWrap::NoWrap {
        true => {
            kids.iter().sum::<f32>() + pixels(node.column_gap) * kids.len().saturating_sub(1) as f32
        }
        false => kids.iter().copied().fold(0.0, f32::max),
    };
    inner + pad
}

/// Returns every text under `entity`, its own first.
fn texts(world: &World, entity: Entity) -> Vec<String> {
    let mut said: Vec<String> = world
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .into_iter()
        .collect();
    if let Some(children) = world.get::<Children>(entity) {
        said.extend(children.into_iter().flat_map(|child| texts(world, *child)));
    }
    said
}

/// Returns a line for each node of `found` that has a fixed width its content does not fit,
/// with the texts under it, so a failure names what overflows.
pub fn overflows(world: &World, found: &[Entity]) -> Vec<String> {
    let over = |entity: &Entity| {
        let Val::Px(fixed) = world.get::<Node>(*entity)?.width else {
            return None;
        };
        let inside = content(world, *entity);
        (inside > fixed + 0.01).then(|| format!("{inside} in {fixed}: {:?}", texts(world, *entity)))
    };
    found.iter().filter_map(over).collect()
}
