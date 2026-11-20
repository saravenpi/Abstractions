//! Flight effects and the values other modules read.
//!
//! Everything drawn here is pooled: the entities, meshes and materials are made once and only
//! moved afterwards. Free entities are `Rebased`, the rush ring is a child of the camera, and
//! the trail ribbons keep their points in the planet frame, so an anchor rebase leaves nothing
//! behind. The wingtip trails and the boot exhaust leave from the limbs of the drawn body
//! ([`crate::avatar::Marks`]); the rest is placed from the gear's own attitude. Every other
//! body marked with a [`Wake`] lays trails of its own the same way.

mod flare;
mod haze;
mod heat;
mod kit;
mod lay;
mod motes;
mod ribbon;
mod scene;
mod streaks;
mod trail;
mod trails;
mod under;
mod wakes;
mod warm;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_exhaust;
#[cfg(test)]
mod tests_fx;
#[cfg(test)]
mod tests_trail;

use bevy::prelude::*;

pub use wakes::Wake;

use super::mode::Mode;
use crate::screens::{Screen, WarmUp, playing};

/// When the gear systems run, in `Update`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GearSet {
    /// [`FlightFx`] is written from the [`super::Gear`] of the player. Step the gear before it.
    Publish,
    /// The effects are drawn from it.
    Draw,
}

/// What the flight feels like right now, for the camera, the audio and the companion.
///
/// All zero and [`Mode::Off`] while nobody is on foot.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct FlightFx {
    /// Speed through the air in metres per second.
    pub airspeed: f32,
    /// Felt load in g, eased: 1 in steady flight, more in pull-ups and tight turns.
    pub load: f32,
    /// Booster push from 0 to 1, thinning with the air.
    pub thrust: f32,
    /// Working share of the wing from 0 (folded) to 1.
    pub wings: f32,
    /// What the gear is doing.
    pub mode: Mode,
    /// How hard the cloth of the wingsuit flutters, from 0 to 1.
    pub flutter: f32,
    /// How far the rush of air closes in on the view at very high speed, from 0 to 1.
    pub rush: f32,
}

/// Returns true while the game is on its gameplay screen with no menu over it and the boot
/// screen gone. An app without those states, as in a test of the effects alone, always plays.
pub fn live(
    screen: Option<Res<State<Screen>>>,
    menu: Option<Res<State<crate::menus::Menu>>>,
    warm: Option<Res<State<WarmUp>>>,
) -> bool {
    let shown = screen.is_none_or(|screen| *screen.get() == Screen::Gameplay);
    let free = menu.is_none_or(|menu| *menu.get() == crate::menus::Menu::None);
    shown && free && warm.is_none_or(|warm| *warm.get() == WarmUp::Idle)
}

/// Registers the effects: their resources and order, then their systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FlightFx>()
        .init_resource::<scene::Scene>()
        .configure_sets(
            Update,
            (GearSet::Publish, GearSet::Draw)
                .chain()
                .after(crate::avatar::AvatarSet::Animate),
        )
        .add_systems(Startup, kit::build)
        .add_systems(Update, warm::show.run_if(in_state(WarmUp::Hidden)))
        .add_systems(OnExit(WarmUp::Hidden), warm::hide);
    systems(app);
}

/// Registers the systems that spawn, publish and draw the effects.
fn systems(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (trails::spawn, streaks::spawn, motes::spawn, flare::spawn),
    )
    .add_systems(
        Update,
        (
            (flare::equip, haze::fit),
            (live.pipe(scene::observe), heat::offer.run_if(heat::fast)).chain(),
        )
            .in_set(GearSet::Publish),
    )
    .add_systems(
        Update,
        (
            trails::draw,
            (wakes::spawn, wakes::draw).chain(),
            streaks::draw,
            haze::draw,
            flare::draw,
            motes::step,
        )
            .in_set(GearSet::Draw)
            .run_if(playing),
    );
}
