//! Drawing the pool of markers: each node moved to its pixel and dressed for what it stands
//! for, the rest hidden. A name that would cross a marker or a name drawn before it is left
//! out, so names stay readable where markers crowd. Only what changed is written.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::level::Level;
use super::pins::{DOT, POOL, PinDot, PinLabel, PinSlot, Pins, pixel};
use super::surface::Surface;
use crate::computer::{GLYPH, clear_of_bands};
use crate::computer::{MapView, SIZE};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::ui::theme;

/// How many times larger the selected marker of a planet is drawn.
pub const CHOSEN: f32 = 1.75;

/// The look of one marker: its fill, its turn and its name. Markers have no outline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dress<'a> {
    /// Colour inside the dot.
    pub fill: Color,
    /// Turn of the dot in radians: a waypoint is a square on its corner.
    pub turn: f32,
    /// The name written beside it.
    pub name: &'a str,
}

/// Returns how a marker of `kind` is drawn: in its own colour or white, grey or bright,
/// and turned onto its corner, with the name it has without a label.
pub fn shape(kind: MarkerKind) -> (bool, bool, f32, &'static str) {
    match kind {
        MarkerKind::Friend => (true, false, 0.0, "Player"),
        MarkerKind::Waypoint => (true, false, std::f32::consts::FRAC_PI_4, "Waypoint"),
        MarkerKind::Favourite => (false, true, 0.0, "Favourite"),
        MarkerKind::Home => (false, false, 0.0, "Home"),
        MarkerKind::Ship => (false, true, 0.0, "Ship"),
    }
}

/// Returns the look of `marker`, or of the player for nothing: the player white, a friend and a
/// waypoint in their colour, a favourite and the ship grey squares, home white.
pub fn dress(marker: Option<&Marker>) -> Dress<'_> {
    let Some(marker) = marker else {
        let white = (false, false, 0.0, "You");
        return look(white, Color::WHITE, "");
    };
    look(shape(marker.kind), marker.color, &marker.label)
}

/// Returns the look of a marker drawn as `shape` says, in `color` when it has its own, named
/// `label` or the name of its shape.
fn look<'a>(
    (own, grey, turn, name): (bool, bool, f32, &'static str),
    color: Color,
    label: &'a str,
) -> Dress<'a> {
    let tint = [theme::TEXT, color][usize::from(own)];
    Dress {
        fill: [tint, theme::MUTED][usize::from(grey)],
        turn,
        name: [label, name][usize::from(label.is_empty())],
    }
}

/// Where a marker is drawn, in pixels of the map, and how many characters its name has.
pub type Spot = Option<(Vec2, usize)>;

/// Returns which markers show their name on a map `side` pixels high: a name is left out when
/// it would cross the dot or the shown name of a marker before it, or run under the header or
/// footer words. The player comes first, so their name shows wherever there is room.
pub fn named(spots: &[Spot; POOL], side: f32) -> [bool; POOL] {
    let dot = |at: Vec2| Rect::from_center_size(at, Vec2::splat(DOT));
    let name = |(at, chars): (Vec2, usize)| {
        let corner = at + Vec2::new(DOT, -0.6 * theme::FONT_SMALL);
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
    /// Their dots.
    dots: Query<
        'w,
        's,
        (
            &'static PinDot,
            &'static mut BackgroundColor,
            &'static mut UiTransform,
        ),
    >,
    /// Their names.
    labels: Query<'w, 's, (&'static PinLabel, &'static mut Text)>,
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

/// Fills, turns and sizes the dot of each node of the pool as `look` says, the node `chosen`
/// larger than the others.
fn paint<'a>(pool: &mut Pool, look: &impl Fn(usize) -> Option<Dress<'a>>, chosen: Option<usize>) {
    for (PinDot(index), mut fill, mut turn) in &mut pool.dots {
        let Some(dress) = look(*index) else {
            continue;
        };
        fill.set_if_neq(BackgroundColor(dress.fill));
        let rotation = Rot2::radians(dress.turn);
        let scale = Vec2::splat([1.0, CHOSEN][usize::from(chosen == Some(*index))]);
        if (turn.rotation, turn.scale) != (rotation, scale) {
            (turn.rotation, turn.scale) = (rotation, scale);
        }
    }
}

/// Draws every marker of [`Pins`] on the level shown, with the names [`named`] keeps. The
/// selected marker of a planet is drawn larger.
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
    let chosen = surface.chosen.filter(|_| *level != Level::Chart);
    moved(&mut pool, |index| spots[index].map(|(at, _)| at));
    paint(&mut pool, &look, chosen);
    for (PinLabel(index), mut text) in &mut pool.labels {
        let wanted = look(*index).map(|dress| ["", dress.name][usize::from(shown[*index])]);
        if let Some(name) = wanted.filter(|name| text.0 != *name) {
            text.0 = name.to_string();
        }
    }
}
