//! The wheel, the trackpad and the pan keys on the flat map of a planet: zooming around the
//! pointer from the whole planet to the ground, panning, and looking at the player again.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use crate::computer::{Hand, RESET_KEY, Typing, ZOOM_STEP, keyed, turn};
use crate::map::geo::{copy_near, to_chart};
use crate::map::level::{Level, framed};
use crate::map::me::Me;
use crate::map::roam::{scaled, slid};
use crate::map::surface::Surface;

/// What the wheel reads: the scroll and the pinch.
type Scroll<'w> = (Res<'w, AccumulatedMouseScroll>, Res<'w, Hand>);

/// Zooms around the pointer with the wheel within what [`framed`] allows. The wheel never
/// leaves the planet.
pub(crate) fn wheel((scroll, hand): Scroll, mut surface: ResMut<Surface>) {
    let notches = hand.notches(turn(&scroll) * f32::from(u8::from(surface.over)));
    if surface.over && notches != 0.0 {
        surface.goal = scaled(surface.goal, surface.at, ZOOM_STEP.powf(notches));
    }
}

/// Looks at the player again on the reset key, unless something is typed.
pub(crate) fn recentre(
    (keys, typing): (Res<ButtonInput<KeyCode>>, Res<Typing>),
    (me, level): (Me, Res<Level>),
    mut surface: ResMut<Surface>,
) {
    let Level::Planet(body) = *level else {
        return;
    };
    if !(keys.just_pressed(RESET_KEY) && typing.is_off()) {
        return;
    }
    let here = me.spot().filter(|(on, _)| *on == body);
    let x = surface.goal.centre.x;
    surface.goal.centre = here.map_or(Vec2::ZERO, |(_, at)| copy_near(to_chart(at), x));
    surface.goal = framed(surface.goal);
}

/// What the pan keys read.
type Keys<'w> = (
    Res<'w, Time>,
    Res<'w, ButtonInput<KeyCode>>,
    Res<'w, Typing>,
);

/// Moves the flat map by the drift a drag left and by the pan keys held, unless a search or a
/// name is typed.
pub(crate) fn coast(
    (time, keys, typing): Keys,
    mut hand: ResMut<Hand>,
    mut surface: ResMut<Surface>,
) {
    if surface.held.is_some() {
        return;
    }
    let held = [Vec2::ZERO, keyed(&keys, surface.over)][usize::from(typing.is_off())];
    let step = hand.coast(held, time.delta_secs());
    if step != Vec2::ZERO {
        surface.lens = slid(surface.lens, step);
        surface.goal = slid(surface.goal, step);
    }
}
