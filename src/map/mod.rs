//! The map: one screen that zooms from the ground under the player to the whole universe.
//!
//! - [`explored`] keeps the ground each body has had uncovered and shares it on a server.
//! - [`raster`] paints the flat coloured picture of a body, black where nobody has been.
//! - The levels: the flat map of a planet, then the charts of the board computer for a
//!   system, a galaxy and the universe. The wheel goes from one to the next, and each fades in.
//! - The markers: the player, friends, waypoints, favourites, home and the ship.
//! - The side panel lists the favourites and the players beside the search of the computer.

pub mod explored;
pub mod palette;
pub mod raster;

mod action;
mod fade;
mod favourite;
mod geo;
#[cfg(test)]
mod geo_tests;
mod glance;
mod level;
mod link;
mod me;
mod pins;
mod pins_draw;
mod planet;
mod side;
mod side_act;
mod side_show;
mod spots;
mod surface;
#[cfg(test)]
mod tests;
mod waypoint;

pub use fade::veil;
pub use favourite::Favourites;
pub use geo::{coords as place_words, from_chart, lat_lon, to_chart};
pub use level::{Level, charting, on_planet};
pub use link::{from_wire, marks_of, marks_text, to_wire};
pub use pins::pin_layer;
pub use planet::{caption, coords, planet_layer};
pub use side::{favourites, players};
pub use surface::Surface;
pub use waypoint::{Waypoint, WaypointChanged, Waypoints, toggle};

use bevy::prelude::*;

use crate::computer::Computer;
use crate::menus::MarkerSet;
use crate::net::{Link, Receive};
use crate::screens::{in_control, playing};

/// Run condition: the map screen is open.
fn open(state: Option<Res<State<Computer>>>) -> bool {
    state.is_some_and(|state| *state.get() == Computer::Open)
}

/// Registers the systems of the open screen: the flat map of a planet, the markers, the side
/// panel and the fade.
fn screen(app: &mut App) {
    let planet = (
        glance::aim,
        glance::press,
        glance::wheel,
        surface::ease,
        surface::slide,
    )
        .chain()
        .run_if(on_planet);
    let shown = (
        surface::enter.run_if(resource_changed::<Level>),
        planet,
        pins::collect,
        pins_draw::draw,
        planet::show,
        planet::picture,
        planet::caption_text,
        planet::coords_text,
        side_show::write,
        fade::fade,
    );
    app.add_systems(OnEnter(Computer::Open), level::open)
        .add_observer(side_act::on_activate)
        .add_systems(Update, shown.chain().after(MarkerSet::Show).run_if(open));
}

/// Registers the map.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Level>()
        .init_resource::<Surface>()
        .init_resource::<pins::Pins>()
        .init_resource::<Waypoints>()
        .init_resource::<Favourites>()
        .add_message::<WaypointChanged>()
        .add_plugins((explored::plugin, raster::plugin, waypoint::plugin, screen))
        .add_systems(
            Update,
            (
                (favourite::load, favourite::store).chain(),
                (waypoint::fill, favourite::fill).in_set(MarkerSet::Fill),
                waypoint::drop_key.run_if(playing),
                favourite::name_keys.run_if(open),
                action::engage.run_if(in_control),
            ),
        )
        .add_systems(
            Update,
            (
                link::receive.after(Receive),
                link::send.after(waypoint::drop_key),
            )
                .run_if(resource_exists::<Link>),
        );
}
