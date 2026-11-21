//! Laying the compass out every frame: which markers it shows, where and how faded, written
//! into a pool of marks that grows when more markers show than it holds. Every mark sits on one
//! level.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::aim::Around;
use super::stack::{extra, phrase, stack, width};
use super::{Bar, Far, Glyph, HEIGHT, Pin, pin};
use crate::menus::hud::context::{Where, marker_shown};
use crate::menus::hud::needle::{WIDTH, fade, slot};
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
    /// The row it stands in, 0 for the bar.
    pub row: usize,
    /// Its width in pixels, icon and words in their strip.
    pub wide: f32,
    /// How many marks that found no row hide behind it.
    pub more: usize,
}

/// Returns the icon of a marker of `kind`.
pub fn icon(kind: MarkerKind) -> Icon {
    match kind {
        MarkerKind::Home => Icon::Home,
        MarkerKind::Ship => Icon::ShipParked,
        MarkerKind::Friend => Icon::User,
        MarkerKind::Waypoint => Icon::MapPin,
        MarkerKind::Favourite => Icon::MapPin,
        MarkerKind::Herd => Icon::Animal,
        MarkerKind::Course => Icon::Go,
    }
}

/// Returns the name a marker of `kind` shows when it has no label of its own.
pub(super) const fn kind_name(kind: MarkerKind) -> &'static str {
    match kind {
        MarkerKind::Home => "Home",
        MarkerKind::Ship => "Ship",
        MarkerKind::Friend => "Player",
        MarkerKind::Waypoint | MarkerKind::Favourite => "Pin",
        MarkerKind::Herd => "Herd",
        MarkerKind::Course => "Course",
    }
}

/// The label of a mark: its slot, its words and their colour.
type Words = (&'static Far, &'static mut Text, &'static mut TextColor);

/// The nodes of the compass.
#[derive(SystemParam)]
pub struct Pins<'w, 's> {
    /// Spawns marks when the pool is too small.
    commands: Commands<'w, 's>,
    /// The bar and its visibility.
    bars: Query<'w, 's, (Entity, &'static mut Visibility), With<Bar>>,
    /// The marks of the pool.
    pins: Query<
        'w,
        's,
        (
            &'static mut Pin,
            &'static mut Node,
            &'static mut BackgroundColor,
        ),
        Without<Bar>,
    >,
    /// The icons of the marks.
    glyphs:
        Query<'w, 's, (&'static Glyph, &'static mut Text, &'static mut TextColor), Without<Far>>,
    /// The labels of the marks.
    words: Query<'w, 's, Words, Without<Glyph>>,
}

impl Pins<'_, '_> {
    /// Shows the bar, empty or not, and grows the pool to `wanted`.
    fn frame(&mut self, wanted: usize) {
        let held = self.pins.iter().count();
        for (bar, mut visibility) in &mut self.bars {
            visibility.set_if_neq(Visibility::Inherited);
            for slot in held..wanted {
                self.commands.entity(bar).with_child(pin(slot));
            }
        }
    }

    /// Moves every mark of the pool to the marker it shows, or hides it.
    fn place(&mut self, seen: &[Seen], markers: &[Marker]) {
        for (mut pin, mut node, mut fill) in &mut self.pins {
            let mark = seen.get(pin.slot);
            let kind = mark.map(|mark| markers[mark.index].kind);
            pin.set_if_neq(Pin { kind, ..*pin });
            let display = mark.map_or(Display::None, |_| Display::Flex);
            let (left, top, wide) = mark.map_or((node.left, node.top, node.width), |mark| {
                (
                    px(mark.at - mark.wide * 0.5),
                    px(mark.row as f32 * HEIGHT),
                    px(mark.wide),
                )
            });
            let back = match mark {
                Some(mark) if mark.row > 0 => theme::PANEL,
                _ => Color::NONE,
            };
            if (node.display, node.left, node.top, node.width) != (display, left, top, wide) {
                (node.display, node.left, node.top, node.width) = (display, left, top, wide);
            }
            fill.set_if_neq(BackgroundColor(back));
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
    /// a waypoint or a favourite, under its icon.
    fn label(&mut self, seen: &[Seen], markers: &[Marker], words: &mut String) {
        for (far, mut text, mut color) in &mut self.words {
            let Some(mark) = seen.get(far.0) else {
                continue;
            };
            let marker = &markers[mark.index];
            phrase(marker, mark.gap, words);
            if mark.more > 0 {
                words.push_str(&extra(mark.more));
            }
            if text.0 != *words {
                text.0.clone_from(words);
            }
            let tint = match marker.kind {
                MarkerKind::Home | MarkerKind::Ship => theme::TEXT,
                _ => marker.color,
            };
            color.set_if_neq(TextColor(tint.with_alpha(tint.alpha() * mark.alpha)));
        }
    }
}

/// Shows every marker that lies inside the span of the bar where it lies, faded toward the
/// ends, all on one level. The bar shows even while empty.
pub fn update(
    around: Around,
    (markers, now): (Res<Markers>, Res<Where>),
    mut pins: Pins,
    mut seen: Local<Vec<Seen>>,
    mut words: Local<String>,
) {
    seen.clear();
    for (index, marker) in markers.0.iter().enumerate() {
        if !marker_shown(now.0, marker.kind) {
            continue;
        }
        let Some((bearing, gap)) = around.reading(marker) else {
            continue;
        };
        if let Some(alpha) = fade(bearing) {
            let at = slot(bearing) * WIDTH;
            phrase(marker, gap, &mut words);
            let wide = width(&words);
            seen.push(Seen {
                index,
                at,
                alpha,
                gap,
                row: 0,
                wide,
                more: 0,
            });
        }
    }
    stack(&mut seen);
    pins.frame(seen.len());
    pins.place(&seen, &markers.0);
    pins.paint(&seen, &markers.0);
    pins.label(&seen, &markers.0, &mut words);
}
