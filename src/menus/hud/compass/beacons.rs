//! The game's own markers: the home base and the ship, rewritten every frame for the compass,
//! the minimap and the map.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::home::Base;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::ship::Ship;
use crate::space::{Anchor, Frame};
use crate::universe::Universe;

/// Owner of the markers of the game itself.
const GAME: u32 = 0;

/// Returns a white marker of the game, without a label.
fn game(kind: MarkerKind, place: MarkerPlace) -> Marker {
    Marker {
        kind,
        place,
        color: Color::WHITE,
        label: String::new(),
        owner: GAME,
    }
}

/// Writes the home marker: the base once it is found, else the home planet itself while the
/// player is away from it.
pub fn home(
    base: Option<Res<Base>>,
    universe: Option<Res<Universe>>,
    anchor: Option<Res<Anchor>>,
    mut markers: ResMut<Markers>,
) {
    markers.clear(MarkerKind::Home, GAME);
    let planet = universe.map(|universe| universe.home());
    let away = anchor.is_some_and(|anchor| planet.map(Frame::Surface) != Some(anchor.frame));
    let place = match base.as_deref() {
        Some(base) => Some(MarkerPlace::Body(base.body, base.centre)),
        None => planet
            .filter(|_| away)
            .map(|planet| MarkerPlace::Body(planet, DVec3::ZERO)),
    };
    if let Some(place) = place {
        markers.0.push(game(MarkerKind::Home, place));
    }
}

/// Writes the ship marker: on its planet while it stands in a surface frame, else in the
/// anchor frame while it shares it.
pub fn ship(ships: Query<&Ship>, anchor: Option<Res<Anchor>>, mut markers: ResMut<Markers>) {
    markers.clear(MarkerKind::Ship, GAME);
    let frame = anchor.map(|anchor| anchor.frame);
    let place = ships.iter().next().and_then(|ship| match ship.frame {
        Frame::Surface(body) => Some(MarkerPlace::Body(body, ship.pos)),
        shared if Some(shared) == frame => Some(MarkerPlace::Anchor(ship.pos)),
        _ => None,
    });
    if let Some(place) = place {
        markers.0.push(game(MarkerKind::Ship, place));
    }
}
