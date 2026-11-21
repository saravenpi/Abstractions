//! Markers over the world: the course, the pin the player follows, friends, home, the ship on
//! foot and the pins within reach, drawn where they really are. In view, the icon sits at the
//! projected place of the target with its name and its distance under it. Out of view, it
//! slides to the border of the safe rectangle with an arrow toward it.
//!
//! One rule set, written down with its numbers and sources in `handoff/specs/markers.md`: a
//! marker whose target is near is reduced, then hidden, because the thing is its own sign
//! ([`rule`]); it fades in the middle of the view; it keeps out of the room of the rest of the
//! HUD ([`zones`]); and two that meet are settled by rank ([`overlap`]).
//!
//! One source of truth: the shared [`Markers`] list the compass and the minimap read. A pool of
//! [`MAX`] nodes is spawned once and refilled every frame without allocating. Nothing shows in
//! the map, in menus or when the setting is off.

mod draw;
mod overlap;
mod pick;
mod place;
mod pool;
mod rule;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_face;
#[cfg(test)]
mod tests_overlap;
#[cfg(test)]
mod tests_rule;
#[cfg(test)]
mod tests_zones;
mod tune;
mod zones;

use place::{MAX, place};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::compass::{Around, icon};
use super::context::{Context, Element, Where};
use super::fade::gate;
use crate::menus::{Marker, Markers};
use crate::settings::{Persistent, Settings};
use pick::State;
use pool::Nodes;
use rule::Face;

/// How fast an icon follows its target, per second.
const FOLLOW: f32 = 25.0;

/// What one frame needs besides the markers: the settings, the scale of the interface, the
/// time and the window.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct View<'w, 's> {
    markers: Res<'w, Markers>,
    settings: Res<'w, Persistent<Settings>>,
    scale: Option<Res<'w, UiScale>>,
    time: Res<'w, Time>,
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
}

/// The numbers of one frame: the screen in interface pixels, the field of view, how far an icon
/// moves toward its target, the rectangle icons stay in and where the player is.
#[derive(Debug, Clone, Copy)]
struct Lens {
    screen: Vec2,
    fov: f32,
    follow: f32,
    safe: Rect,
    context: Context,
}

impl View<'_, '_> {
    /// Returns the numbers of this frame in `context`.
    fn lens(&self, context: Context) -> Lens {
        let scale = self.scale.as_ref().map_or(1.0, |scale| scale.0).max(0.1);
        let screen = Vec2::new(self.window.width(), self.window.height()) / scale;
        Lens {
            screen,
            fov: (self.settings.graphics.fov as f32).to_radians(),
            follow: 1.0 - (-FOLLOW * self.time.delta_secs()).exp(),
            safe: zones::safe(screen),
            context,
        }
    }
}

/// The layer of markers over the world, full screen, with its pool of hidden slots.
pub(super) fn layer() -> impl Bundle {
    (
        gate(Element::Markers),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Pickable::IGNORE,
        pool::slots(),
    )
}

/// What is written into a slot: where its icon is, its arrow and how it is drawn.
type Drawn = (Vec2, Option<f32>, Face);

/// Writes slot `at`: the icon of `marker`, its arrow and its `words`.
fn paint(nodes: &mut Nodes, at: usize, marker: &Marker, drawn: Drawn, words: &str) {
    let (pos, arrow, face) = drawn;
    let side = draw::side(face);
    let color = draw::tint(marker).with_alpha(face.glyph);
    draw::room(nodes, at, Some((pos, side)));
    draw::icon(nodes, at, icon(marker.kind), (color, side));
    draw::arrow(nodes, at, arrow.filter(|_| side > 0.0), color);
    draw::label(nodes, at, words, face.label);
}

/// Draws slot `at`: places the marker it shows, works out how near, how central and how
/// crowded it is, and writes it, or hides it.
fn show(
    at: usize,
    (state, view, lens): (&mut State, &View, Lens),
    (nodes, taken): (&mut Nodes, &mut Vec<Rect>),
) {
    let Some(pick) = state.chosen(at) else {
        state.memory[at].who = None;
        draw::room(nodes, at, None);
        return;
    };
    let marker = &view.markers.0[pick.index];
    let safe = lens.safe.inflate(-state.stickiness(at));
    let placed = place(pick.dir, lens.fov, lens.screen, safe);
    let pos = state.settle(at, (marker.kind, pick.index), placed, lens.follow);
    let pos = zones::fit(pos, lens.screen, lens.context.is_ship());
    let look = state.look(at, marker, pick.gap, lens.context);
    let angle = (-pick.dir.z).clamp(-1.0, 1.0).acos();
    let dim = placed.arrow.is_none().then(|| rule::centre(angle));
    let wanted = rule::face(marker.kind, look, dim);
    let Some(face) = overlap::settle(wanted, pos, taken) else {
        draw::room(nodes, at, None);
        return;
    };
    let far = (look.detail >= 0.5).then_some(pick.gap);
    draw::say(&mut state.words, (draw::name(marker), far));
    paint(nodes, at, marker, (pos, placed.arrow, face), &state.words);
}

/// Chooses the markers to show, places each and writes the pool.
pub(super) fn update(
    (around, now): (Around, Res<Where>),
    view: View,
    (mut state, mut taken): (Local<State>, Local<Vec<Rect>>),
    mut nodes: Nodes,
) {
    let level = view.settings.gameplay.markers();
    state.gather(&around, &view.markers, (now.0, level));
    let lens = view.lens(now.0);
    taken.clear();
    for at in 0..MAX {
        show(at, (&mut *state, &view, lens), (&mut nodes, &mut taken));
    }
}

/// Run condition: markers over the world mean something where the player is. The setting is
/// read by `update`, which hides every slot when it is off.
fn enabled(now: Res<Where>) -> bool {
    now.has(Element::Markers)
}

/// Registers the markers over the world, drawn after the markers are filled and the compass
/// read them.
pub(super) fn plugin(app: &mut App) {
    use super::MarkerSet;
    use crate::screens::{Screen, in_control};
    app.add_systems(
        Update,
        update
            .after(MarkerSet::Fill)
            .run_if(in_control)
            .run_if(enabled)
            .run_if(in_state(Screen::Gameplay)),
    );
}
