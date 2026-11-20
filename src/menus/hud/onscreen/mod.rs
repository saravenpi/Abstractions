//! Markers over the world: the active pin, the course, home, the ship on foot, friends and the
//! pins within reach, drawn where they really are. In view, the icon sits at the projected
//! place of the target with its distance under it. Out of view, it slides to the border of the
//! screen with an arrow toward it, at the height of the real position.
//!
//! One source of truth: the shared [`Markers`] list the compass and the minimap read. A pool of
//! [`MAX`] nodes is spawned once and refilled every frame without allocating. Nothing shows in
//! the map, in menus or when the setting is off.

mod draw;
mod pick;
mod place;
mod pool;
#[cfg(test)]
mod tests;

use place::{MAX, dimmed, place};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::compass::{Around, icon};
use super::context::{Element, Where};
use super::fade::gate;
use crate::menus::Markers;
use crate::settings::{Persistent, Settings};
use crate::ui::theme;
use pick::State;
use pool::Nodes;

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
/// moves toward its target and how far in from the screen edge icons stay.
#[derive(Debug, Clone, Copy)]
struct Lens {
    screen: Vec2,
    fov: f32,
    follow: f32,
    inset: f32,
}

impl View<'_, '_> {
    /// Returns the numbers of this frame.
    fn lens(&self) -> Lens {
        let scale = self.scale.as_ref().map_or(1.0, |scale| scale.0).max(0.1);
        Lens {
            screen: Vec2::new(self.window.width(), self.window.height()) / scale,
            fov: (self.settings.graphics.fov as f32).to_radians(),
            follow: 1.0 - (-FOLLOW * self.time.delta_secs()).exp(),
            inset: theme::EDGE + theme::ICON * 0.5,
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

/// Draws slot `at`: places the marker it shows, or hides it.
fn show(at: usize, (state, view, lens): (&mut State, &View, Lens), nodes: &mut Nodes) {
    let Some(pick) = state.chosen(at) else {
        state.memory[at].who = None;
        draw::room(nodes, at, None);
        return;
    };
    let marker = &view.markers.0[pick.index];
    let inset = lens.inset + state.stickiness(at);
    let placed = place(pick.dir, lens.fov, lens.screen, inset);
    let pos = state.settle(at, (marker.kind, pick.index), placed, lens.follow);
    let on_screen = placed.arrow.is_none();
    let dim = if on_screen {
        dimmed((-pick.dir.z).clamp(-1.0, 1.0).acos())
    } else {
        1.0
    };
    let color = pick::tint(marker).with_alpha(dim);
    draw::room(nodes, at, Some(pos));
    draw::icon(nodes, at, icon(marker.kind), color);
    draw::arrow(nodes, at, placed.arrow, color);
    draw::label(nodes, at, (pick.gap, dim), &mut state.words);
}

/// Chooses the markers to show, places each and writes the pool.
pub(super) fn update(
    (around, now): (Around, Res<Where>),
    view: View,
    mut state: Local<State>,
    mut nodes: Nodes,
) {
    state.gather(&around, &view.markers, now.0);
    let lens = view.lens();
    for at in 0..MAX {
        show(at, (&mut *state, &view, lens), &mut nodes);
    }
}

/// Run condition: the player turned markers over the world on.
fn enabled(settings: Res<Persistent<Settings>>, now: Res<Where>) -> bool {
    settings.gameplay.world_markers && now.has(Element::Markers)
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
