//! The in-game HUD: flight instruments in the ship, the one prompt line, the location and
//! presence banner, the compass that points at every marker, the coordinates and climate
//! lines, the minimap and the markers over the world. Everything hides behind menus and the
//! computer.
//!
//! What shows is one rule: `context::shown` maps where the player is to the elements that
//! mean something there, and every element asks it. One edge margin, `theme::EDGE`, frames
//! the screen: top left the place, top centre the compass, top right the minimap, bottom left
//! the instruments and frame, bottom centre the prompt line.

mod banner;
mod climate;
mod compass;
mod context;
mod coords;
mod course;
mod eye;
mod fade;
mod flight;
mod frame;
mod hints;
mod markers;
mod minimap;
mod needle;
mod onscreen;
mod panel;
mod prompt;
mod snow;
mod switch;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_banner;
#[cfg(test)]
mod tests_climate;
#[cfg(test)]
mod tests_context;
#[cfg(test)]
mod tests_coords;
#[cfg(test)]
mod tests_deck;
#[cfg(test)]
mod tests_frame;
#[cfg(test)]
mod tests_hints;
#[cfg(test)]
mod tests_icons;
#[cfg(test)]
mod tests_pin_hint;
#[cfg(test)]
mod tests_prompt;
#[cfg(test)]
mod tests_rows;
#[cfg(test)]
mod tests_shown;
#[cfg(test)]
mod tests_size;
#[cfg(test)]
mod tests_status;
mod units;

pub use climate::HudWeather;
pub use markers::{Marker, MarkerKind, MarkerPlace, MarkerSet, Markers};

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::condition::Condition;
use crate::screens::{Screen, in_control};
use crate::settings::{Persistent, Settings};
use crate::ui::theme;

/// Seconds between two refreshes of the flight instruments and the prompt line.
const REFRESH: f32 = 0.1;

/// Seconds between two refreshes of the coordinates line.
const COORDS_EVERY: f32 = 0.1;

/// Seconds between two refreshes of the climate line.
const CLIMATE_EVERY: f32 = 0.5;

/// Seconds between two redraws of the static over the view.
const SNOW_EVERY: f32 = 0.05;

/// Marks the root of the HUD.
#[derive(Component)]
struct HudRoot;

/// The bottom-left stack: the instrument block over the frame rows, so they never overlap.
fn stack() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(theme::EDGE),
            bottom: px(theme::EDGE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: px(theme::S2),
            ..default()
        },
        Pickable::IGNORE,
        children![panel::block(), frame::corner()],
    )
}

/// Spawns the hidden HUD for the world being entered.
fn spawn(mut commands: Commands) {
    commands.insert_resource(banner::Banner::default());
    commands.spawn((
        HudRoot,
        DespawnOnExit(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Pickable::IGNORE,
        Visibility::Hidden,
        children![
            stack(),
            prompt::line(),
            banner::block(),
            compass::bar(),
            coords::corner(),
            minimap::frame(),
            onscreen::layer(),
            snow::screen()
        ],
    ));
}

/// Shows the HUD while the player controls the game.
fn show(In(control): In<bool>, mut roots: Query<&mut Visibility, With<HudRoot>>) {
    let wanted = match control {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    };
    for mut visibility in &mut roots {
        visibility.set_if_neq(wanted);
    }
}

/// Registers the frame corner and the static of a hurt head.
fn hurt(app: &mut App) {
    let snow = on_timer(Duration::from_secs_f32(SNOW_EVERY));
    app.init_resource::<Condition>().add_systems(
        Update,
        (
            frame::update.run_if(resource_changed::<Condition>.or_else(frame::fresh)),
            snow::update.run_if(snow),
        )
            .run_if(in_control)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// The steps of the HUD in a frame, in order.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Step {
    /// Where the player is is worked out.
    Context,
    /// The elements read the world while the player controls the game.
    Playing,
    /// The banner announces and fades.
    Banner,
    /// Elements that entered or left their context fade.
    Fade,
}

/// The elements that read the world while the player controls the game.
fn readouts(app: &mut App) {
    let every = || on_timer(Duration::from_secs_f32(REFRESH));
    let place = coords::update
        .run_if(context::shows_coords)
        .run_if(on_timer(Duration::from_secs_f32(COORDS_EVERY)));
    let marks = (
        compass::update.run_if(context::shows_compass),
        compass::north,
        place,
    );
    let weather = climate::update
        .run_if(context::shows_climate)
        .run_if(on_timer(Duration::from_secs_f32(CLIMATE_EVERY)));
    let playing = (
        hints::update,
        (panel::refresh, panel::spool, switch::write).run_if(every()),
        prompt::update.run_if(every()),
        marks.in_set(MarkerSet::Show),
        weather,
        banner::watch,
    );
    app.add_systems(Update, playing.chain().in_set(Step::Playing));
}

/// Registers the steps and the systems that are not elements.
fn steps(app: &mut App) {
    app.configure_sets(
        Update,
        (
            Step::Context,
            Step::Playing.run_if(in_control),
            Step::Banner,
            Step::Fade,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    )
    .add_systems(
        Update,
        (in_control.pipe(show), in_control.pipe(context::update)).in_set(Step::Context),
    )
    .add_systems(
        Update,
        (banner::announce, banner::fade.run_if(banner::showing))
            .chain()
            .in_set(Step::Banner),
    )
    .add_systems(Update, fade::step.in_set(Step::Fade));
}

/// Registers the HUD systems.
pub(super) fn plugin(app: &mut App) {
    markers::plugin(app);
    eye::plugin(app);
    compass::plugin(app);
    hurt(app);
    app.init_resource::<banner::Banner>()
        .init_resource::<context::Where>()
        .init_resource::<hints::Hinted>()
        .init_resource::<hints::Clock>()
        .add_plugins((minimap::plugin, onscreen::plugin))
        .add_systems(OnEnter(Screen::Gameplay), spawn)
        .add_systems(
            Update,
            hints::retire
                .run_if(resource_exists::<Persistent<Settings>>)
                .run_if(resource_exists_and_changed::<crate::map::Places>),
        );
    steps(app);
    readouts(app);
}
