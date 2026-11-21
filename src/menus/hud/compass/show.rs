//! Laying the compass out every frame: which markers it shows, where and how faded, written
//! into a pool of marks that grows when more markers show than it holds. Every mark is an icon
//! on the row of the bar with its distance on the line under it.

use std::fmt::Write;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::aim::Around;
use super::space::space;
use super::{Bar, Count, Far, Glyph, LANE, Pin, pin};
use crate::menus::hud::context::{Where, marker_shown};
use crate::menus::hud::needle::{WIDTH, fade, slot};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::ui::units::distance_short;
use crate::ui::{Icon, theme};

/// One marker the compass shows this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seen {
    /// Its index in [`Markers`].
    pub index: usize,
    /// Its place in pixels along the bar.
    pub at: f32,
    /// How opaque it is, from 0 to 1: faded toward the ends of the bar, dimmed behind another.
    pub alpha: f32,
    /// Its distance in metres.
    pub gap: f64,
    /// How many friends past the limit of the bar fold into it.
    pub more: usize,
    /// Width in pixels of its distance on the bar, with the gap that keeps two apart.
    pub wide: f32,
    /// Whether its distance is written under it: not when it would print over another.
    pub said: bool,
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

/// The count of a mark: its slot, its words and their colour.
type Words = (&'static Count, &'static mut Text, &'static mut TextColor);

/// The distance under a mark: its slot, its words and their colour.
type Distances = (&'static Far, &'static mut Text, &'static mut TextColor);

/// The nodes of the compass.
#[derive(SystemParam)]
pub struct Pins<'w, 's> {
    /// Spawns marks when the pool is too small.
    commands: Commands<'w, 's>,
    /// The bar and its visibility.
    bars: Query<'w, 's, (Entity, &'static mut Visibility), With<Bar>>,
    /// The marks of the pool.
    pins: Query<'w, 's, (&'static mut Pin, &'static mut Node), Without<Bar>>,
    /// The icons of the marks.
    glyphs: Query<
        'w,
        's,
        (&'static Glyph, &'static mut Text, &'static mut TextColor),
        (Without<Count>, Without<Far>),
    >,
    /// The counts of the marks.
    words: Query<'w, 's, Words, (Without<Glyph>, Without<Far>)>,
    /// The distances under the marks.
    far: Query<'w, 's, Distances, (Without<Glyph>, Without<Count>)>,
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
        for (mut pin, mut node) in &mut self.pins {
            let mark = seen.get(pin.slot);
            let kind = mark.map(|mark| markers[mark.index].kind);
            pin.set_if_neq(Pin { kind, ..*pin });
            let display = mark.map_or(Display::None, |_| Display::Flex);
            let left = mark.map_or(node.left, |mark| px(mark.at - LANE * 0.5));
            if (node.display, node.left) != (display, left) {
                (node.display, node.left) = (display, left);
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

    /// Writes the count of every shown mark that folds friends into it, faded, in its colour,
    /// and nothing for the others.
    fn count(&mut self, seen: &[Seen], markers: &[Marker], words: &mut String) {
        for (count, mut text, mut color) in &mut self.words {
            let Some(mark) = seen.get(count.0) else {
                continue;
            };
            words.clear();
            if mark.more > 0 {
                let _ = write!(words, "+{}", mark.more);
            }
            if text.0 != *words {
                text.0.clone_from(words);
            }
            let tint = markers[mark.index].color;
            color.set_if_neq(TextColor(tint.with_alpha(tint.alpha() * mark.alpha)));
        }
    }

    /// Writes the distance under every shown mark, short, in the secondary tone and faded with
    /// its mark, and nothing under a mark whose distance would print over another.
    fn say(&mut self, seen: &[Seen], words: &mut String) {
        for (far, mut text, mut color) in &mut self.far {
            let Some(mark) = seen.get(far.0) else {
                continue;
            };
            words.clear();
            if mark.said {
                words.push_str(&distance_short(mark.gap));
            }
            if text.0 != *words {
                text.0.clone_from(words);
            }
            let tone = theme::TEXT_2;
            color.set_if_neq(TextColor(tone.with_alpha(tone.alpha() * mark.alpha)));
        }
    }
}

/// Shows every marker that lies inside the span of the bar where it lies, faded toward the
/// ends, each with its distance under it. The bar shows even while empty.
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
            seen.push(Seen {
                index,
                at,
                alpha,
                gap,
                more: 0,
                wide: 0.0,
                said: true,
            });
        }
    }
    space(&mut seen, &markers);
    pins.frame(seen.len());
    pins.place(&seen, &markers.0);
    pins.paint(&seen, &markers.0);
    pins.count(&seen, &markers.0, &mut words);
    pins.say(&seen, &mut words);
}
