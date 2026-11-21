//! Test rig: the height each node of a built screen takes, as the flex layout of the interface
//! stacks it. A box that scrolls gives up its height; a box set to a least height of its own,
//! or one that clips, shrinks down to what its children need at least; any other box takes all
//! it holds, as a flex item does by default.

use bevy::prelude::*;
use bevy::text::{FontSize, LineBreak};
use bevy::ui::OverflowAxis;

use super::rig::{flowing, pixels};
use super::stage::lines;
use crate::ui::text_width;

/// Height of a line of text as a share of its size: the default line height of Bevy.
const LINE: f32 = 1.2;

/// How the children of a node are measured.
type Measure = fn(&World, Entity, f32) -> f32;

/// Returns the width a wrapping text in `entity` wraps in: the fixed width of the nearest box
/// around it less the paddings on the way, or `screen` less them when no box has one.
fn wrap_room(world: &World, entity: Entity, screen: f32) -> f32 {
    let mut padding = 0.0;
    let mut at = Some(entity);
    while let Some(here) = at {
        if let Some(node) = world.get::<Node>(here) {
            padding += pixels(node.padding.left) + pixels(node.padding.right);
            if let Val::Px(width) = node.width {
                return width - padding;
            }
        }
        at = world.get::<ChildOf>(here).map(ChildOf::parent);
    }
    screen - padding
}

/// Returns the height the text of `entity` takes, or nothing when it has no text: one line a
/// paragraph, or the lines it wraps to in the room around it.
fn text_height(world: &World, entity: Entity, screen: f32) -> Option<f32> {
    let (text, font) = (world.get::<Text>(entity)?, world.get::<TextFont>(entity)?);
    let FontSize::Px(size) = font.font_size else {
        return None;
    };
    let wraps = world
        .get::<TextLayout>(entity)
        .is_none_or(|layout| layout.linebreak != LineBreak::NoWrap);
    let room = wrap_room(world, entity, screen);
    let space = text_width(" ", size, 0.0);
    let count = |paragraph: &str| -> usize {
        let words: Vec<f32> = match wraps {
            true => paragraph
                .split_whitespace()
                .map(|word| text_width(word, size, 0.0))
                .collect(),
            false => Vec::new(),
        };
        lines(&words, space, room).max(1)
    };
    let all: usize = text.0.split('\n').map(count).sum();
    Some(all as f32 * LINE * size)
}

/// Returns the height the children of `entity` take, each measured by `each` with its margins:
/// added up in a column, the tallest in a row. The padding of `entity` is added.
fn stack(world: &World, entity: Entity, screen: f32, each: Measure) -> f32 {
    let node = world.get::<Node>(entity).expect("a node");
    let outer = |child: Entity| {
        let margin = world.get::<Node>(child).expect("a node").margin;
        each(world, child, screen) + pixels(margin.top) + pixels(margin.bottom)
    };
    let kids: Vec<f32> = flowing(world, entity).into_iter().map(outer).collect();
    let column = matches!(
        node.flex_direction,
        FlexDirection::Column | FlexDirection::ColumnReverse
    );
    let inner = match column {
        true => {
            kids.iter().sum::<f32>() + pixels(node.row_gap) * kids.len().saturating_sub(1) as f32
        }
        false => kids.iter().copied().fold(0.0, f32::max),
    };
    inner + pixels(node.padding.top) + pixels(node.padding.bottom)
}

/// Returns the height `entity` takes with all it holds: its fixed height, else its text or its
/// children, within its least and its most height.
pub fn full(world: &World, entity: Entity, screen: f32) -> f32 {
    let node = world.get::<Node>(entity).expect("a node");
    if let Val::Px(fixed) = node.height {
        return fixed;
    }
    let pad = pixels(node.padding.top) + pixels(node.padding.bottom);
    let held = text_height(world, entity, screen)
        .map_or_else(|| stack(world, entity, screen, full), |text| text + pad);
    let most = match node.max_height {
        Val::Px(most) => most,
        _ => f32::INFINITY,
    };
    held.min(most).max(pixels(node.min_height))
}

/// Returns the least height `entity` takes when a height is set on it from outside, as on a
/// screen or in a row that stretches it: nothing when it scrolls, else its text, or what its
/// children take at least in a column, or the tallest of them in a row.
pub fn inside(world: &World, entity: Entity, screen: f32) -> f32 {
    let node = world.get::<Node>(entity).expect("a node");
    if let Val::Px(fixed) = node.height {
        return fixed;
    }
    if node.overflow.y == OverflowAxis::Scroll {
        return pixels(node.min_height);
    }
    let pad = pixels(node.padding.top) + pixels(node.padding.bottom);
    let column = matches!(
        node.flex_direction,
        FlexDirection::Column | FlexDirection::ColumnReverse
    );
    let each: Measure = if column { least } else { inside };
    let held = text_height(world, entity, screen)
        .map_or_else(|| stack(world, entity, screen, each), |text| text + pad);
    held.max(pixels(node.min_height))
}

/// Returns the least height `entity` takes in a column that is short of room: a fixed height
/// stays, a box that does not shrink takes all it holds, one that scrolls nothing, one with a
/// least height of its own or one that clips what it needs [`inside`], any other all it holds.
pub fn least(world: &World, entity: Entity, screen: f32) -> f32 {
    let node = world.get::<Node>(entity).expect("a node");
    let set = matches!(node.height, Val::Px(_)) || node.flex_shrink == 0.0;
    let yields = matches!(node.min_height, Val::Px(_)) || node.overflow.y != OverflowAxis::Visible;
    match (set, yields) {
        (false, true) => inside(world, entity, screen),
        _ => full(world, entity, screen),
    }
}
