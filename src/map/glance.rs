//! The pointer and the wheel on the flat map of a planet: pointing at a marker or a place,
//! dragging the map, zooming it, and rising to the system chart past the whole planet.
//!
//! A click picks the place under the pointer, or the marker on it; a double click sets the
//! course to the planet. The reset key looks at the player again.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::geo::{copy_near, from_chart, to_chart, wrapped};
use super::level::{Level, WHOLE, framed, rise};
use super::me::Me;
use super::pins::Pins;
use super::surface::Surface;
use crate::computer::{
    Area, Board, DOUBLE_CLICK, Destination, Field, MapView, REACH, RESET_KEY, SIZE, SLOP, Target,
    Typing, ZOOM_STEP,
};
use crate::ui::theme;

/// The square map and the room around it, which say where the pointer is.
type Under<'w, 's> = (
    Query<'w, 's, &'static RelativeCursorPosition, With<Field>>,
    Query<'w, 's, &'static RelativeCursorPosition, With<Area>>,
);

/// Returns the marker of `pins` drawn nearest to the pixel `at` within the reach of the
/// pointer, through `surface`'s lens.
fn nearest(pins: &Pins, surface: &Surface, at: Vec2) -> Option<usize> {
    let lens = surface.lens;
    let gap = |index: usize| {
        lens.to_map(copy_near(pins.0[index].at, lens.centre.x))
            .distance(at)
    };
    let near = (0..pins.0.len()).filter(|index| gap(*index) <= REACH);
    near.min_by(|a, b| gap(*a).total_cmp(&gap(*b)))
}

/// Follows the pointer on the planet: the place under it and the marker it is on. The chart
/// is told the pointer is over the map, so the wheel does not scroll the results.
pub(super) fn aim(
    (fields, areas): Under,
    pins: Res<Pins>,
    mut surface: ResMut<Surface>,
    mut view: ResMut<MapView>,
) {
    let share = fields.iter().next().and_then(|field| field.normalized);
    let over = areas.iter().next().is_some_and(|area| area.cursor_over);
    let at = share.map_or(surface.at, |share| (share + 0.5) * SIZE);
    if (at, over) == (surface.at, surface.over) && !pins.is_changed() {
        return;
    }
    let point = from_chart(wrapped(surface.lens.to_chart(at))).filter(|_| over);
    let hover = nearest(&pins, &surface, at).filter(|_| over && surface.held.is_none());
    (surface.at, surface.over, surface.point, surface.hover) = (at, over, point, hover);
    view.over = over;
}

/// Applies one click at `now` seconds and returns true when it was the second on the same
/// thing within the time of a double click.
fn tap(surface: &mut Surface, now: f32) -> bool {
    let last = surface.clicked.replace((surface.hover, now));
    surface.picked = surface.point.or(surface.picked);
    let again = last.is_some_and(|(mark, then)| mark == surface.hover && now - then < DOUBLE_CLICK);
    if again {
        surface.clicked = None;
    }
    again
}

/// Drags the map while the left button is held from it, and on a release that did not drag,
/// clicks: a double click sets the course to the planet shown.
pub(super) fn press(
    (buttons, time, level): (Res<ButtonInput<MouseButton>>, Res<Time>, Res<Level>),
    mut surface: ResMut<Surface>,
    mut course: ResMut<Destination>,
) {
    if buttons.just_pressed(MouseButton::Left) && surface.over {
        (surface.held, surface.dragged) = (Some(surface.at), false);
    }
    let Some(from) = surface.held else {
        return;
    };
    if buttons.pressed(MouseButton::Left) {
        if surface.dragged || from.distance(surface.at) > SLOP {
            surface.lens = framed(surface.lens.panned(surface.at - from));
            surface.goal = surface.lens;
            (surface.held, surface.dragged) = (Some(surface.at), true);
        }
        return;
    }
    surface.held = None;
    if let Level::Planet(body) = *level
        && !surface.dragged
        && tap(&mut surface, time.elapsed_secs())
    {
        course.0 = Some(Target::Body(body));
    }
}

/// What the wheel reads besides the planet: the keys, the player and the chart it rises to.
type Wheel<'w, 's> = (
    Res<'w, AccumulatedMouseScroll>,
    Res<'w, ButtonInput<KeyCode>>,
    Res<'w, Typing>,
    Me<'w, 's>,
);

/// Zooms around the pointer with the wheel within what [`framed`] allows, looks at the player
/// again on the reset key, and rises to the system chart when the wheel turns out of the whole
/// planet.
pub(super) fn wheel(
    (scroll, keys, typing, me): Wheel,
    mut surface: ResMut<Surface>,
    mut level: ResMut<Level>,
    (board, mut view): (Res<Board>, ResMut<MapView>),
) {
    let Level::Planet(body) = *level else {
        return;
    };
    if keys.just_pressed(RESET_KEY) && typing.is_off() {
        let here = me.spot().filter(|(on, _)| *on == body);
        let x = surface.goal.centre.x;
        surface.goal.centre = here.map_or(Vec2::ZERO, |(_, at)| copy_near(to_chart(at), x));
        surface.goal = framed(surface.goal);
    }
    let notches = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / theme::ROW_HEIGHT,
    };
    if !surface.over || notches == 0.0 {
        return;
    }
    if notches < 0.0 && surface.goal.zoom <= WHOLE {
        *level = Level::Chart;
        if let Some((lens, goal)) = rise(&view, &board, body) {
            (view.lens, view.goal) = (lens, goal);
        }
        return;
    }
    let factor = ZOOM_STEP.powf(notches);
    surface.goal = framed(surface.goal.zoomed(surface.at, factor));
}
