//! Clicks and drags on the chart: a click selects a mark or lets go of it, a double click
//! eases onto it and goes into its place, and a drag pans the chart under the pointer, which
//! drifts on a little once let go.

use bevy::prelude::*;

use super::{DOUBLE_CLICK, MapView, SLOP};
use crate::computer::board::Board;
use crate::computer::chart::Dot;
use crate::computer::coast::keyed;
use crate::computer::hand::Hand;
use crate::computer::lens::Lens;
use crate::computer::route::Target;
use crate::computer::semantic::DIVE;
use crate::computer::tab::Tab;
use crate::computer::typing::Typing;

/// Returns the zoom that frames `target` on the chart of `tab`: the whole system for its
/// star, and the dive zoom for anything else, from which one more notch on it goes inside.
pub fn framing(tab: Tab, target: Target) -> f32 {
    match (tab, target) {
        (Tab::System, Target::Star(_)) => 1.0,
        _ => DIVE,
    }
}

/// Returns the lens that focuses the map on `dot`: centred on it, at the zoom that frames it.
/// Nothing when the chart has no such dot.
pub fn aimed(board: &Board, view: &MapView, dot: Dot) -> Option<Lens> {
    let (_, at) = view.dots.iter().find(|(other, _)| *other == dot)?;
    let target = match dot {
        Dot::Row(index) => board.lines.get(index).map(|line| line.target),
        _ => None,
    };
    let zoom = target.map_or(DIVE, |target| framing(board.tab, target));
    Some(Lens { zoom, centre: *at }.clamped())
}

/// Applies one click at `now` seconds. A click on the mark of a row selects that row. A click
/// on empty map or on the ship lets go of the selection. A double click eases the map onto the
/// mark and asks to go into its place, or shows the whole chart on empty map. No click sets the
/// course.
pub(crate) fn tap(view: &mut MapView, board: &mut Board, now: f32) {
    let last = view.clicked.replace((view.hover, now));
    let same = last.is_some_and(|(mark, _)| mark == view.hover);
    if same && last.is_some_and(|(_, then)| now - then < DOUBLE_CLICK) {
        let whole = view.hover.is_none().then(Lens::default);
        let focus = view.hover.and_then(|dot| aimed(board, view, dot));
        view.goal = focus.or(whole).unwrap_or(view.goal);
        view.clicked = None;
        if let Some(Dot::Row(index)) = view.hover {
            view.into = board.lines.get(index).map(|line| line.target);
        }
        return;
    }
    match view.hover {
        Some(Dot::Row(index)) => {
            board.step(index as isize - board.cursor as isize);
            (view.picked, view.marked) = (true, true);
        }
        _ => (view.picked, view.marked) = (false, false),
    }
}

/// What a press of the pointer reads: its buttons and the time.
type Buttons<'w> = (Res<'w, ButtonInput<MouseButton>>, Res<'w, Time>);

/// Drags the chart while the left button is held from the map, so the point under the pointer
/// stays under it, and clicks on a release that did not drag.
pub(crate) fn press(
    (buttons, time): Buttons,
    mut board: ResMut<Board>,
    mut view: ResMut<MapView>,
    mut hand: ResMut<Hand>,
) {
    if buttons.just_pressed(MouseButton::Left) && view.over {
        (view.held, view.dragged, hand.drift) = (Some(view.at), false, Vec2::ZERO);
    }
    let Some(from) = view.held else {
        return;
    };
    if buttons.pressed(MouseButton::Left) {
        if view.dragged || from.distance(view.at) > SLOP {
            let moved = view.at - from;
            view.lens = view.lens.panned(moved);
            view.goal = view.lens;
            (view.held, view.dragged) = (Some(view.at), true);
            hand.follow(moved, time.delta_secs());
        }
        return;
    }
    view.held = None;
    if !view.dragged {
        hand.drift = Vec2::ZERO;
        tap(&mut view, &mut board, time.elapsed_secs());
    }
}

/// What the keys that pan the chart read.
type Keys<'w> = (
    Res<'w, Time>,
    Res<'w, ButtonInput<KeyCode>>,
    Option<Res<'w, Typing>>,
);

/// Moves the chart by the drift a drag left and by the pan keys held, unless a search or a
/// name is typed.
pub(crate) fn coast((time, keys, typing): Keys, mut hand: ResMut<Hand>, mut view: ResMut<MapView>) {
    if view.held.is_some() {
        return;
    }
    let free = typing.is_none_or(|typing| typing.is_off());
    let held = [Vec2::ZERO, keyed(&keys, view.over)][usize::from(free)];
    let step = hand.coast(held, time.delta_secs());
    if step != Vec2::ZERO {
        view.lens = view.lens.panned(step);
        view.goal = view.goal.panned(step);
    }
}
