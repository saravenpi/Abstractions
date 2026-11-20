//! Writing one slot of the pool: its place, its icon, its arrow and its distance.

use std::fmt::Write;

use bevy::prelude::*;

use super::pool::{Nodes, ROOM};
use crate::computer::distance;
use crate::ui::{Icon, theme};

/// Pixels from the centre of an icon to the centre of its arrow.
const ARROW_REACH: f32 = 22.0;

/// Moves the room of slot `at` so its icon is centred on `pos`, or hides it for nothing.
pub fn room(nodes: &mut Nodes, at: usize, pos: Option<Vec2>) {
    for (slot, mut node) in &mut nodes.0 {
        if slot.0 != at {
            continue;
        }
        let Some(pos) = pos else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let (left, top) = (px(pos.x - ROOM * 0.5), px(pos.y - theme::ICON * 0.5));
        if (node.display, node.left, node.top) != (Display::Flex, left, top) {
            (node.display, node.left, node.top) = (Display::Flex, left, top);
        }
    }
}

/// Writes the icon of slot `at`: `symbol` in `color`.
pub fn icon(nodes: &mut Nodes, at: usize, symbol: Icon, color: Color) {
    for (glyph, mut text, mut paint) in &mut nodes.1 {
        if glyph.0 != at {
            continue;
        }
        if !text.0.starts_with(symbol.codepoint()) {
            text.0 = symbol.glyph();
        }
        paint.set_if_neq(TextColor(color));
    }
}

/// Writes the arrow of slot `at`: turned toward `angle` radians clockwise from up, or hidden.
pub fn arrow(nodes: &mut Nodes, at: usize, angle: Option<f32>, color: Color) {
    for (arrow, mut node, mut turn, mut paint) in &mut nodes.2 {
        if arrow.0 != at {
            continue;
        }
        let display = [Display::None, Display::Flex][usize::from(angle.is_some())];
        if node.display != display {
            node.display = display;
        }
        if let Some(angle) = angle {
            let (x, y) = (angle.sin() * ARROW_REACH, -angle.cos() * ARROW_REACH);
            turn.set_if_neq(UiTransform {
                translation: Val2::px(x, y),
                rotation: Rot2::radians(angle),
                ..UiTransform::IDENTITY
            });
        }
        paint.set_if_neq(TextColor(color));
    }
}

/// Writes the distance of `gap` metres under the icon of slot `at` at opacity `alpha`, using
/// `words` as scratch.
pub fn label(nodes: &mut Nodes, at: usize, (gap, alpha): (f64, f32), words: &mut String) {
    words.clear();
    let _ = write!(words, "{}", distance(gap));
    for (label, mut text, mut paint) in &mut nodes.3 {
        if label.0 != at {
            continue;
        }
        if text.0 != *words {
            text.0.clone_from(words);
        }
        paint.set_if_neq(TextColor(theme::TEXT.with_alpha(alpha)));
    }
}
