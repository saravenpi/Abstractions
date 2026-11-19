//! Laying the compass out every frame: which markers it shows, where, how faded and on which
//! row, written into a pool of marks that grows when more markers show than it holds.

use std::fmt::Write;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::aim::Around;
use super::{Bar, Far, Glyph, HEIGHT, Pin, ROOM, ROW, pin};
use crate::computer::distance;
use crate::menus::hud::needle::{ROWS, WIDTH, fade, row, slot};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::ui::{Icon, theme};

/// One marker the compass shows this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seen {
    /// Its index in [`Markers`].
    pub index: usize,
    /// Its place in pixels along the bar.
    pub at: f32,
    /// How opaque it is, from 0 to 1.
    pub alpha: f32,
    /// Its distance in metres.
    pub gap: f64,
    /// The row it sits on, from 0 at the bar.
    pub row: usize,
}

/// Sorts the shown markers from left to right and gives each the first row where it covers no
/// other mark.
pub fn arrange(seen: &mut [Seen]) {
    seen.sort_unstable_by(|a, b| a.at.total_cmp(&b.at).then(a.index.cmp(&b.index)));
    let mut ends = [f32::NEG_INFINITY; ROWS];
    for mark in seen.iter_mut() {
        mark.row = row(mark.at, &mut ends);
    }
}

/// Returns the icon of a marker of `kind`.
fn icon(kind: MarkerKind) -> Icon {
    match kind {
        MarkerKind::Home => Icon::Home,
        MarkerKind::Ship => Icon::Send,
        MarkerKind::Friend => Icon::User,
        MarkerKind::Waypoint => Icon::MapPin,
        MarkerKind::Favourite => Icon::Star,
    }
}

/// The nodes of the compass.
#[derive(SystemParam)]
pub struct Pins<'w, 's> {
    /// Spawns marks when the pool is too small.
    commands: Commands<'w, 's>,
    /// The bar, its visibility and its height.
    bars: Query<'w, 's, (Entity, &'static mut Visibility, &'static mut Node), With<Bar>>,
    /// The marks of the pool.
    pins: Query<'w, 's, (&'static mut Pin, &'static mut Node), Without<Bar>>,
    /// The icons of the marks.
    glyphs:
        Query<'w, 's, (&'static Glyph, &'static mut Text, &'static mut TextColor), Without<Far>>,
    /// The labels of the marks.
    words: Query<'w, 's, (&'static Far, &'static mut Text, &'static mut TextColor), Without<Glyph>>,
}

impl Pins<'_, '_> {
    /// Shows the bar while it has marks, as high as their rows, and grows the pool to `wanted`.
    fn frame(&mut self, wanted: usize, rows: usize) {
        let held = self.pins.iter().count();
        for (bar, mut visibility, mut node) in &mut self.bars {
            let shown = [Visibility::Hidden, Visibility::Inherited][usize::from(wanted > 0)];
            visibility.set_if_neq(shown);
            let height = px(HEIGHT + ROW * rows.saturating_sub(1) as f32);
            if node.height != height {
                node.height = height;
            }
            for slot in held..wanted {
                self.commands.entity(bar).with_child(pin(slot));
            }
        }
    }

    /// Moves every mark of the pool to the marker it shows, or hides it.
    fn place(&mut self, seen: &[Seen], markers: &[Marker]) {
        for (mut pin, mut node) in &mut self.pins {
            let mark = seen.get(pin.slot);
            let kind = mark.map(|mark| markers[mark.index].kind);
            pin.set_if_neq(Pin { kind, ..*pin });
            let display = mark.map_or(Display::None, |_| Display::Flex);
            let (left, top) = mark.map_or((node.left, node.top), |mark| {
                (px(mark.at - ROOM * 0.5), px(ROW * mark.row as f32))
            });
            if (node.display, node.left, node.top) != (display, left, top) {
                (node.display, node.left, node.top) = (display, left, top);
            }
        }
    }

    /// Writes the icon of every shown mark, in its colour and faded.
    fn paint(&mut self, seen: &[Seen], markers: &[Marker]) {
        for (glyph, mut text, mut color) in &mut self.glyphs {
            let Some(mark) = seen.get(glyph.0) else {
                continue;
            };
            let marker = &markers[mark.index];
            let symbol = icon(marker.kind);
            if !text.0.starts_with(symbol.codepoint()) {
                text.0 = symbol.glyph();
            }
            color.set_if_neq(TextColor(marker.color.with_alpha(mark.alpha)));
        }
    }

    /// Writes the name and distance of every shown mark, faded, in its colour for a friend,
    /// a waypoint or a favourite.
    fn label(&mut self, seen: &[Seen], markers: &[Marker], words: &mut String) {
        for (far, mut text, mut color) in &mut self.words {
            let Some(mark) = seen.get(far.0) else {
                continue;
            };
            let marker = &markers[mark.index];
            words.clear();
            let gap = distance(mark.gap);
            let _ = match marker.label.is_empty() {
                true => write!(words, "{gap}"),
                false => write!(words, "{}  {gap}", marker.label),
            };
            if text.0 != *words {
                text.0.clone_from(words);
            }
            let tint = match marker.kind {
                MarkerKind::Home | MarkerKind::Ship => theme::MUTED,
                _ => marker.color,
            };
            color.set_if_neq(TextColor(tint.with_alpha(tint.alpha() * mark.alpha)));
        }
    }
}

/// Shows every marker that lies inside the span of the bar where it lies, faded toward the
/// ends and stacked in rows, and hides the bar while none does.
pub fn update(
    around: Around,
    markers: Res<Markers>,
    mut pins: Pins,
    mut seen: Local<Vec<Seen>>,
    mut words: Local<String>,
) {
    seen.clear();
    for (index, marker) in markers.0.iter().enumerate() {
        let Some((bearing, gap)) = around.reading(marker) else {
            continue;
        };
        if let Some(alpha) = fade(bearing) {
            let (at, row) = (slot(bearing) * WIDTH, 0);
            seen.push(Seen {
                index,
                at,
                alpha,
                gap,
                row,
            });
        }
    }
    arrange(&mut seen);
    let rows = seen.iter().map(|mark| mark.row + 1).max().unwrap_or(1);
    pins.frame(seen.len(), rows);
    pins.place(&seen, &markers.0);
    pins.paint(&seen, &markers.0);
    pins.label(&seen, &markers.0, &mut words);
}
