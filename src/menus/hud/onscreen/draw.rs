//! Writing one slot of the pool: its place, its icon, its arrow and its words.

use std::fmt::Write;

use bevy::prelude::*;
use bevy::text::FontSize;

use super::pool::{Nodes, ROOM};
use super::rule::Face;
use super::tune::ARROW_REACH;
use crate::menus::{Marker, MarkerKind};
use crate::ui::units::distance;
use crate::ui::{Icon, theme};

/// What the words of a marker say: its name, empty for none, and its distance in metres when
/// the distance is worth reading.
pub type Said<'a> = (&'a str, Option<f64>);

/// Returns the colour a marker is drawn in: the accent for the course, the pin colour for pins,
/// its own for friends, white for home and the ship.
pub fn tint(marker: &Marker) -> Color {
    match marker.kind {
        MarkerKind::Course => theme::ACCENT,
        MarkerKind::Waypoint | MarkerKind::Favourite => theme::PIN,
        MarkerKind::Friend => marker.color,
        _ => theme::TEXT,
    }
}

/// Returns the name the words of `marker` start with: its own, or what its icon stands for.
pub fn name(marker: &Marker) -> &str {
    match marker.kind {
        MarkerKind::Home => "Home",
        MarkerKind::Ship => "Ship",
        _ => marker.label.as_str(),
    }
}

/// Returns the side in pixels of the icon of a marker drawn as `face`: nothing while the icon
/// is not drawn, so that the words take its place.
pub fn side(face: Face) -> f32 {
    match (face.glyph > 0.0, face.small) {
        (false, _) => 0.0,
        (true, true) => theme::ICON_SMALL,
        (true, false) => theme::ICON,
    }
}

/// Moves the room of slot `at` so an icon `side` pixels large is centred on `pos`, or its words
/// when it has no icon, or hides it for nothing.
pub fn room(nodes: &mut Nodes, at: usize, spot: Option<(Vec2, f32)>) {
    for (slot, mut node) in &mut nodes.0 {
        if slot.0 != at {
            continue;
        }
        let Some((pos, side)) = spot else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let above = [theme::LINE_SMALL, side][usize::from(side > 0.0)] * 0.5;
        let (left, top) = (px(pos.x - ROOM * 0.5), px(pos.y - above));
        if (node.display, node.left, node.top) != (Display::Flex, left, top) {
            (node.display, node.left, node.top) = (Display::Flex, left, top);
        }
    }
}

/// Writes the icon of slot `at`: `symbol` in `color`, `side` pixels large, or no icon at all
/// for a side of nothing.
pub fn icon(nodes: &mut Nodes, at: usize, symbol: Icon, (color, side): (Color, f32)) {
    for (glyph, mut text, mut paint, mut font, mut node) in &mut nodes.1 {
        if glyph.0 != at {
            continue;
        }
        let display = [Display::None, Display::Flex][usize::from(side > 0.0)];
        if node.display != display {
            node.display = display;
        }
        if !text.0.starts_with(symbol.codepoint()) {
            text.0 = symbol.glyph();
        }
        if side > 0.0 && font.font_size != FontSize::Px(side) {
            font.font_size = FontSize::Px(side);
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

/// Writes into `words` what `said` reads as: "Ana", "340 m" or "Ana 340 m".
pub fn say(words: &mut String, (name, gap): Said) {
    words.clear();
    words.push_str(name);
    if let Some(gap) = gap {
        let space = ["", " "][usize::from(!name.is_empty())];
        let _ = write!(words, "{space}{}", distance(gap));
    }
}

/// Writes the words of slot `at` at opacity `alpha`, on their strip, or hides them when they
/// are empty or not to be seen.
pub fn label(nodes: &mut Nodes, at: usize, words: &str, alpha: f32) {
    for (label, mut text, mut paint, mut strip, mut node) in &mut nodes.3 {
        if label.0 != at {
            continue;
        }
        let shown = alpha > 0.0 && !words.is_empty();
        let display = [Display::None, Display::Flex][usize::from(shown)];
        if node.display != display {
            node.display = display;
        }
        if shown && text.0 != words {
            words.clone_into(&mut text.0);
        }
        paint.set_if_neq(TextColor(theme::TEXT.with_alpha(alpha)));
        let fill = theme::PANEL.with_alpha(theme::PANEL.alpha() * alpha);
        strip.set_if_neq(BackgroundColor(fill));
    }
}
