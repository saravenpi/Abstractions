//! The map: one screen that shows the ground under the player and, a level at a time, out to
//! the whole universe.
//!
//! - [`explored`] keeps the ground each body has had uncovered and shares it on a server.
//! - [`raster`] paints the flat coloured picture of a body, clear where nobody has been.
//! - The levels: the flat map of a planet, then the charts of the board computer for a
//!   system, a galaxy and the universe, one at a time. A double click goes into a place, the
//!   path over the map comes out of it, and each fades in.
//! - The markers: the player, friends, pins, home and the ship, each drawn as its icon.
//! - The pins: [`Places`] keeps up to eight. The pin key drops one at the feet or removes the
//!   one within ten metres, and a right click on the ground of the map drops one there.
//! - The side panel lists the pins and the players beside the search of the computer, and puts
//!   the buttons of the selected pin and the keys of the map under them.

pub mod explored;
pub mod palette;
pub mod raster;

mod action;
mod caption;
mod deeds;
#[cfg(test)]
mod deeds_tests;
mod fade;
mod geo;
#[cfg(test)]
mod geo_tests;
mod glance;
mod level;
mod link;
mod me;
mod nav;
mod pins;
mod pins_draw;
mod places;
mod planet;
mod roam;
#[cfg(test)]
mod rules_tests;
mod side;
mod side_act;
mod side_buttons;
mod side_show;
#[cfg(test)]
mod side_tests;
mod spots;
mod surface;
#[cfg(test)]
mod tests;

pub use caption::{caption, coords};
pub use deeds::{Button, Offer, deeds, offer};
pub use fade::veil;
pub use geo::{coords as place_words, from_chart, lat_lon, to_chart};
pub use level::{Level, charting, on_planet};
pub use link::{WaypointChanged, from_wire, marks_of, marks_text, to_wire};
pub use nav::{Place, Stop, Trail, path};
pub use pins::pin_layer;
pub use places::Places;
pub use planet::planet_layer;
pub use side::{PINS, PLAYERS, pins, players};
pub use surface::Surface;

use bevy::prelude::*;

use crate::computer::Computer;
use crate::menus::MarkerSet;
use crate::net::{Link, Receive};
use crate::screens::{in_control, playing};

/// Run condition: the map screen is open.
fn open(state: Option<Res<State<Computer>>>) -> bool {
    state.is_some_and(|state| *state.get() == Computer::Open)
}

/// The systems of the flat map of a planet: the pointer, the wheel, the keys and the lens.
fn planet_systems() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    (
        glance::aim,
        glance::press,
        glance::drop_click,
        glance::pick,
        glance::wheel,
        glance::recentre,
        glance::coast,
        surface::ease,
        surface::slide,
    )
        .chain()
        .run_if(on_planet)
}

/// Registers the systems of the open screen: the flat map of a planet, the way between the
/// levels, the markers, the side panel and the fade.
fn screen(app: &mut App) {
    let shown = (
        surface::enter.run_if(resource_changed::<Level>),
        planet_systems(),
        nav::systems(),
        (pins::collect, pins_draw::draw),
        (planet::show, planet::picture, planet::say),
        (caption::caption_text, caption::coords_text),
        (side_show::write, side_show::go_key, deeds::write),
        fade::fade,
    );
    let after = (MarkerSet::Show, crate::computer::Input);
    app.add_systems(OnEnter(Computer::Open), level::open)
        .add_observer(side_act::on_activate)
        .add_systems(
            Update,
            shown.chain().after(after.0).after(after.1).run_if(open),
        );
}

/// Registers what the map keeps between frames.
fn resources(app: &mut App) {
    app.init_resource::<Level>()
        .init_resource::<Surface>()
        .init_resource::<pins::Pins>()
        .init_resource::<Places>()
        .add_message::<WaypointChanged>();
}

/// Registers the map.
pub(super) fn plugin(app: &mut App) {
    let parts = (
        explored::plugin,
        raster::plugin,
        places::plugin,
        nav::plugin,
        screen,
    );
    resources(app);
    app.add_plugins(parts)
        .add_systems(
            Update,
            (
                (places::load, places::store).chain(),
                places::fill.in_set(MarkerSet::Fill),
                places::drop_key.run_if(playing),
                places::arrive.run_if(playing).run_if(places::leading),
                (places::name_keys, side_act::pin_keys).chain().run_if(open),
                action::go.run_if(in_control),
                places::announce,
            ),
        )
        .add_systems(
            Update,
            (
                link::receive.after(Receive),
                link::send.after(places::announce),
            )
                .run_if(resource_exists::<Link>),
        );
}
