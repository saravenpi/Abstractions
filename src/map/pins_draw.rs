//! Drawing the pool of markers: each node moved to its pixel and dressed as the icon of what it
//! stands for, the rest hidden. The point of a pin is the tip of its glyph. A name that would
//! cross a marker or a name drawn before it is left out, so names stay readable where markers
//! crowd, but the selected and the hovered marker always show theirs. Only what changed is
//! written.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::level::Level;
use super::pins::{DOT, GLYPH_SIZE, NAME_GAP, POOL, PinDot, PinLabel, PinSlot, Pins, pixel};
use super::surface::Surface;
use crate::computer::{GLYPH, clear_of_bands};
use crate::computer::{MapView, SIZE};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::ui::{ICON_FONT, Icon, theme};

/// How many times larger the selected marker of a planet is drawn: 24 pixels against 16.
pub const CHOSEN: f32 = 1.5;

/// The look of one marker: its icon, its colour and its name. Markers have no outline and no
/// fill behind the icon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dress<'a> {
    /// The icon.
    pub icon: Icon,
    /// Colour of the icon.
    pub fill: Color,
    /// The name written beside it.
    pub name: &'a str,
}

/// Returns the icon of a marker of `kind` and the name it has without a label.
pub fn shape(kind: MarkerKind) -> (Icon, &'static str) {
    match kind {
        MarkerKind::Friend => (Icon::User, "Player"),
        MarkerKind::Waypoint | MarkerKind::Favourite => (Icon::MapPin, "Pin"),
        MarkerKind::Home => (Icon::Home, "Home"),
        MarkerKind::Ship => (Icon::ShipParked, "Ship"),
        MarkerKind::Herd => (Icon::Animal, "Herd"),
    }
}

/// Returns the look of `marker`, or of the player for nothing: the player, home and the ship
/// white, every other marker in its own colour, which a pin has in the pin colour.
pub fn dress(marker: Option<&Marker>) -> Dress<'_> {
    let Some(marker) = marker else {
        return Dress {
            icon: Icon::User,
            fill: theme::TEXT,
            name: "You",
        };
    };
    let (icon, name) = shape(marker.kind);
    let plain = matches!(marker.kind, MarkerKind::Home | MarkerKind::Ship);
    Dress {
        icon,
        fill: [marker.color, theme::TEXT][usize::from(plain)],
        name: if marker.label.is_empty() {
            name
        } else {
            &marker.label
        },
    }
}

/// Returns `icon` as a glyph of the icon font `size` pixels high in `color`, which no node
/// stands behind.
pub fn glyph(icon: Icon, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(icon.glyph()),
        TextFont::from_font_size(size).with_font(ICON_FONT),
        TextColor(color),
        TextLayout::no_wrap(),
    )
}

/// Where a marker is drawn, in pixels of the map, and how many characters its name has.
pub type Spot = Option<(Vec2, usize)>;

/// Returns which markers show their name on a map `side` pixels high: a name is left out when
/// it would cross the dot or the shown name of a marker before it, or run under the header or
/// footer words. The player comes first, so their name shows wherever there is room.
pub fn named(spots: &[Spot; POOL], side: f32) -> [bool; POOL] {
    let dot = |at: Vec2| Rect::from_center_size(at, Vec2::splat(DOT));
    let name = |(at, chars): (Vec2, usize)| {
        let corner = at + Vec2::new(NAME_GAP, -0.6 * theme::FONT_SMALL);
        let size = Vec2::new(chars as f32 * GLYPH, theme::FONT_SMALL * 1.2);
        Rect::from_corners(corner, corner + size)
    };
    let mut shown = [false; POOL];
    for index in 0..POOL {
        let Some(spot) = spots[index] else {
            continue;
        };
        let words = name(spot);
        let crosses = |rect: Rect| !rect.intersect(words).is_empty();
        let crossed = spots[..index].iter().zip(shown).any(|(other, lit)| {
            other.is_some_and(|other| crosses(dot(other.0)) || (lit && crosses(name(other))))
        });
        shown[index] = !crossed && clear_of_bands(words, side);
    }
    shown
}

/// The nodes of the pool.
#[derive(SystemParam)]
pub struct Pool<'w, 's> {
    /// The nodes moved to the markers.
    slots: Query<'w, 's, (&'static PinSlot, &'static mut Node)>,
    /// Their icons.
    dots: Query<
        'w,
        's,
        (
            &'static PinDot,
            &'static mut Text,
            &'static mut TextColor,
            &'static mut UiTransform,
        ),
        Without<PinLabel>,
    >,
    /// Their names.
    labels: Query<'w, 's, (&'static PinLabel, &'static mut Text), Without<PinDot>>,
}

/// Moves each node of the pool to its marker and hides the nodes left over.
fn moved(pool: &mut Pool, wanted: impl Fn(usize) -> Option<Vec2>) {
    for (PinSlot(index), mut node) in &mut pool.slots {
        let at = wanted(*index);
        let shown = (
            at.map_or(Display::None, |_| Display::Flex),
            at.map_or(auto(), |at| px(at.x)),
            at.map_or(auto(), |at| px(at.y)),
        );
        if (node.display, node.left, node.top) != shown {
            (node.display, node.left, node.top) = shown;
        }
    }
}

/// Sets the icon and the colour of the dot of each node of the pool as `look` says, and sizes
/// it, the node `chosen` larger than the others. A pin is lifted so that its tip, not its
/// middle, is on its point.
fn paint<'a>(pool: &mut Pool, look: &impl Fn(usize) -> Option<Dress<'a>>, chosen: Option<usize>) {
    for (PinDot(index), mut face, mut color, mut place) in &mut pool.dots {
        let Some(dress) = look(*index) else {
            continue;
        };
        face.set_if_neq(Text(dress.icon.glyph()));
        color.set_if_neq(TextColor(dress.fill));
        let scale = [1.0, CHOSEN][usize::from(chosen == Some(*index))];
        let lift = [0.0, 0.5 * GLYPH_SIZE * scale][usize::from(dress.icon == Icon::MapPin)];
        let wanted = UiTransform {
            translation: Val2::px(0.0, -lift),
            scale: Vec2::splat(scale),
            rotation: Rot2::IDENTITY,
        };
        place.set_if_neq(wanted);
    }
}

/// Draws every marker of [`Pins`] on the level shown, with the names [`named`] keeps, and the
/// name of the selected and of the hovered marker besides. The selected marker of a planet is
/// drawn larger.
pub(super) fn draw(
    (pins, markers, level): (Res<Pins>, Res<Markers>, Res<Level>),
    (surface, view): (Res<Surface>, Res<MapView>),
    mut pool: Pool,
) {
    let side = [SIZE, view.side][usize::from(view.side > 0.0)];
    let lenses = (&*surface, &*view);
    let look = |index: usize| {
        pins.0
            .get(index)
            .map(|pin| dress(pin.marker.and_then(|at| markers.0.get(at))))
    };
    let spots: [Spot; POOL] = std::array::from_fn(|index| {
        let at = pixel(*level, lenses, pins.0.get(index)?.at) / SIZE * side;
        look(index).map(|dress| (at, dress.name.chars().count()))
    });
    let shown = named(&spots, side);
    let flat = |pick: Option<usize>| pick.filter(|_| *level != Level::Chart);
    let (chosen, hover) = (flat(surface.chosen), flat(surface.hover));
    moved(&mut pool, |index| spots[index].map(|(at, _)| at));
    paint(&mut pool, &look, chosen);
    for (PinLabel(index), mut text) in &mut pool.labels {
        let lit = shown[*index] || chosen == Some(*index) || hover == Some(*index);
        let wanted = look(*index).map(|dress| ["", dress.name][usize::from(lit)]);
        if let Some(name) = wanted.filter(|name| text.0 != *name) {
            text.0 = name.to_string();
        }
    }
}
