//! Drawing the pool of markers: each node moved to its pixel and dressed for what it stands
//! for, the rest hidden. Only what changed is written.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::level::Level;
use super::pins::{PinDot, PinLabel, PinSlot, Pins, pixel};
use super::surface::Surface;
use crate::computer::{MapView, SIZE};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::ui::theme;

/// The look of one marker: its fill, its border, its turn and its name.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dress<'a> {
    /// Colour inside the dot.
    pub fill: Color,
    /// Colour of the border of the dot.
    pub border: Color,
    /// Turn of the dot in radians: a waypoint is a square on its corner.
    pub turn: f32,
    /// The name written beside it.
    pub name: &'a str,
}

/// Returns how a marker of `kind` is drawn: in its own colour or white, hollow or filled,
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
/// waypoint in their colour, a favourite and the ship hollow squares, home white.
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
    (own, hollow, turn, name): (bool, bool, f32, &'static str),
    color: Color,
    label: &'a str,
) -> Dress<'a> {
    let tint = [theme::TEXT, color][usize::from(own)];
    Dress {
        fill: [tint, Color::NONE][usize::from(hollow)],
        border: [theme::INK, theme::TEXT][usize::from(hollow)],
        turn,
        name: [label, name][usize::from(label.is_empty())],
    }
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
            &'static mut BorderColor,
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

/// Draws every marker of [`Pins`] on the level shown.
pub(super) fn draw(
    (pins, markers, level): (Res<Pins>, Res<Markers>, Res<Level>),
    (surface, view): (Res<Surface>, Res<MapView>),
    mut pool: Pool,
) {
    let side = [SIZE, view.side][usize::from(view.side > 0.0)];
    let lenses = (&*surface, &*view);
    moved(&mut pool, |index| {
        let pin = pins.0.get(index)?;
        Some(pixel(*level, lenses, pin.at) / SIZE * side)
    });
    let look = |index: usize| {
        pins.0
            .get(index)
            .map(|pin| dress(pin.marker.and_then(|at| markers.0.get(at))))
    };
    for (PinDot(index), mut fill, mut border, mut turn) in &mut pool.dots {
        let Some(dress) = look(*index) else {
            continue;
        };
        fill.set_if_neq(BackgroundColor(dress.fill));
        border.set_if_neq(BorderColor::all(dress.border));
        let rotation = Rot2::radians(dress.turn);
        if turn.rotation != rotation {
            turn.rotation = rotation;
        }
    }
    for (PinLabel(index), mut text) in &mut pool.labels {
        if let Some(dress) = look(*index).filter(|dress| text.0 != dress.name) {
            text.0 = dress.name.to_string();
        }
    }
}
