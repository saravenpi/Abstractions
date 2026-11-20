//! The pointer on the flat map of a planet: pointing at a marker or a place, clicking it and
//! dragging the map.
//!
//! A click selects the marker under the pointer, or lets go of the selection on bare ground, and
//! picks the place. A double click focuses the map on the marker or the place. A right click on
//! bare ground drops a pin. No click sets a course.

mod pin;
mod wheel;

pub(super) use pin::{drop_click, pick};
pub(super) use wheel::{coast, recentre, wheel};

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::geo::{copy_near, from_chart, wrapped};
use super::pins::Pins;
use super::roam::{focused, slid};
use super::surface::Surface;
use crate::computer::{Area, DOUBLE_CLICK, Field, Hand, MapView, REACH, SIZE, SLOP};

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

/// Applies one click at `now` seconds: selects the marker under the pointer, lets go of the
/// selection on bare ground, and on a double click focuses the map on the marker of `pins` or
/// on the place under the pointer.
pub fn tap(surface: &mut Surface, pins: &Pins, now: f32) {
    let last = surface.clicked.replace((surface.hover, now));
    surface.picked = surface.point.or(surface.picked);
    let same = last.is_some_and(|(mark, _)| mark == surface.hover);
    if same && last.is_some_and(|(_, then)| now - then < DOUBLE_CLICK) {
        let x = surface.lens.centre.x;
        let pin = surface.hover.and_then(|pin| pins.0.get(pin));
        let at = pin.map_or(surface.lens.to_chart(surface.at), |pin| {
            copy_near(pin.at, x)
        });
        (surface.goal, surface.clicked) = (focused(at, surface.goal.zoom), None);
        return;
    }
    surface.chosen = surface.hover;
}

/// What a press of the pointer reads: its buttons and the time.
type Buttons<'w> = (Res<'w, ButtonInput<MouseButton>>, Res<'w, Time>);

/// Drags the map while the left button is held from it, so the place under the pointer stays
/// under it, and clicks on a release that did not drag.
pub(super) fn press(
    (buttons, time): Buttons,
    pins: Res<Pins>,
    mut surface: ResMut<Surface>,
    mut hand: ResMut<Hand>,
) {
    if buttons.just_pressed(MouseButton::Left) && surface.over {
        (surface.held, surface.dragged, hand.drift) = (Some(surface.at), false, Vec2::ZERO);
    }
    let Some(from) = surface.held else {
        return;
    };
    if buttons.pressed(MouseButton::Left) {
        if surface.dragged || from.distance(surface.at) > SLOP {
            let moved = surface.at - from;
            surface.lens = slid(surface.lens, moved);
            surface.goal = surface.lens;
            (surface.held, surface.dragged) = (Some(surface.at), true);
            hand.follow(moved, time.delta_secs());
        }
        return;
    }
    surface.held = None;
    if !surface.dragged {
        hand.drift = Vec2::ZERO;
        tap(&mut surface, &pins, time.elapsed_secs());
    }
}

#[cfg(test)]
#[path = "../glance_tests.rs"]
mod tests;
