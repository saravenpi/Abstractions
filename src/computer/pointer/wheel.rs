//! The wheel and the trackpad on the chart: zooming around the pointer and panning by the
//! trackpad, always inside the chart shown.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use super::{MapView, RESET_KEY, ZOOM_STEP};
use crate::computer::hand::{Hand, Turn, ctrl, to_map, turn};
use crate::computer::lens::Lens;
use crate::computer::typing::Typing;

/// What the wheel reads: the scroll, the keys and the typing.
type Wheel<'w> = (
    Res<'w, AccumulatedMouseScroll>,
    Res<'w, ButtonInput<KeyCode>>,
    Option<Res<'w, Typing>>,
);

/// Zooms around the pointer with the wheel while the pointer is over the map, within the zoom
/// of the chart shown, pans with the trackpad, and shows the whole chart again on the reset
/// key unless a search is typed. The wheel never changes the chart.
pub(crate) fn zoom((scroll, keys, typing): Wheel, hand: Res<Hand>, mut view: ResMut<MapView>) {
    if keys.just_pressed(RESET_KEY) && typing.is_none_or(|typing| typing.is_off()) {
        view.goal = Lens::default();
    }
    let turned = turn(&scroll, ctrl(&keys)).filter(|_| view.over);
    if let Some(Turn::Pan(pixels)) = turned {
        let moved = to_map(pixels, view.side);
        (view.lens, view.goal) = (view.lens.panned(moved), view.goal.panned(moved));
    }
    let notches = hand.notches(turned);
    if view.over && notches != 0.0 {
        view.goal = view.goal.zoomed(view.at, ZOOM_STEP.powf(notches));
    }
}
