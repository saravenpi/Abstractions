//! The game's own markers: the home base, the ship and the course, rewritten every frame for
//! the compass, the minimap and the map.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::aim::Around;
use crate::computer::Destination;
use crate::home::Base;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::ship::Ship;
use crate::space::{Anchor, Frame};
use crate::ui::theme;
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

/// Writes the home marker: the container of the base once it is found, else the home planet itself while the
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
        Some(base) => Some(MarkerPlace::Body(base.body, base.home())),
        None => planet
            .filter(|_| away)
            .map(|planet| MarkerPlace::Body(planet, DVec3::ZERO)),
    };
    if let Some(place) = place {
        markers.0.push(game(MarkerKind::Home, place));
    }
}

/// Writes the ship marker: on its planet while it stands in a surface frame, else in the
/// anchor frame while it shares it. Skips a local ship currently hidden while another player flies it.
pub fn ship(
    ships: Query<(&Ship, Option<&Visibility>)>,
    anchor: Option<Res<Anchor>>,
    mut markers: ResMut<Markers>,
) {
    markers.clear(MarkerKind::Ship, GAME);
    let frame = anchor.map(|anchor| anchor.frame);
    let place = ships
        .iter()
        .find(|(_, vis)| *vis != Some(&Visibility::Hidden))
        .and_then(|(ship, _)| match ship.frame {
            Frame::Surface(body) => Some(MarkerPlace::Body(body, ship.pos)),
            shared if Some(shared) == frame => Some(MarkerPlace::Anchor(ship.pos)),
            _ => None,
        });
    if let Some(place) = place {
        markers.0.push(game(MarkerKind::Ship, place));
    }
}

/// Writes the course marker: the place the course is set to, at any distance, in the accent
/// colour, so that a set course gives a heading on foot as well as seated.
pub fn course(around: Around, destination: Res<Destination>, mut markers: ResMut<Markers>) {
    markers.clear(MarkerKind::Course, GAME);
    let Some((point, label)) = destination.0.and_then(|target| around.course(target)) else {
        return;
    };
    markers.0.push(Marker {
        kind: MarkerKind::Course,
        place: MarkerPlace::Anchor(point),
        color: theme::ACCENT,
        label,
        owner: GAME,
    });
}
