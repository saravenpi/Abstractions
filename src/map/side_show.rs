//! Writing the lists of the side panel of the map: the pins with how far each is, the one being
//! renamed with a caret, the line under them, the players with their colours. Rows past the end
//! are hidden, and so are the players outside a session.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::me::Me;
use super::places::Places;
use super::side::{
    NoPlayers, PinActions, PinFar, PinGoKey, PinHint, PinName, PinRow, PlayerDot, PlayerName,
    PlayerRow, Players,
};
use crate::computer::{Board, Keys, Typing};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::net::Link;
use crate::saves::Pin;
use crate::settings::{Persistent, Settings};
use crate::ui::Selected;
use crate::ui::units::distance_short;
use crate::universe::{BodyId, Universe};

/// What the line under the pins says while there is no pin.
pub const EMPTY: &str = "No pins yet. Press B or right-click the ground to drop one.";

/// What the line under the pins says while a pin can be selected and none is.
pub const PICK: &str = "Click a pin to select it.";

/// The rows and blocks of the panel that show or hide.
type Rows<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static PinRow>,
        Option<&'static PlayerRow>,
        (Has<Players>, Has<NoPlayers>, Has<PinActions>),
    ),
    Or<(
        With<PinRow>,
        With<PlayerRow>,
        With<Players>,
        With<NoPlayers>,
        With<PinActions>,
    )>,
>;

/// A text of the lists with the one thing that tells it from the other texts.
type Of<'w, 's, M, F> = Query<'w, 's, (&'static M, &'static mut Text), F>;

/// Filters that tell the names of the pins from the other texts.
type NotName = (Without<PinFar>, Without<PinHint>, Without<PlayerName>);

/// Filters that tell the distances of the pins from the other texts.
type NotFar = (Without<PinName>, Without<PinHint>, Without<PlayerName>);

/// Filters that tell the names of the players from the other texts.
type NotPlayer = (Without<PinName>, Without<PinFar>, Without<PinHint>);

/// The texts of the lists.
#[derive(SystemParam)]
pub struct Faces<'w, 's> {
    /// The names of the pins.
    names: Of<'w, 's, PinName, NotName>,
    /// How far each pin is.
    far: Of<'w, 's, PinFar, NotFar>,
    /// The line under the pins.
    hint: Query<'w, 's, &'static mut Text, Hint>,
    /// The names of the players.
    players: Of<'w, 's, PlayerName, NotPlayer>,
    /// The colours of the players.
    dots: Query<'w, 's, (&'static PlayerDot, &'static mut BackgroundColor)>,
}

/// The line under the pins, told from the other texts of the lists.
type Hint = (
    With<PinHint>,
    Without<PinName>,
    Without<PinFar>,
    Without<PlayerName>,
);

/// Returns the name shown for the pin at `index`: what is typed, with a caret, while it is
/// renamed.
fn shown(places: &Places, typing: &Typing, index: usize) -> Option<String> {
    let pin = places.pins.get(index)?;
    Some(match typing {
        Typing::Name(at, name) if *at == index => format!("{name}_"),
        _ => pin.name.clone(),
    })
}

/// Returns the line under the pins for `count` pins and the selected one: the planet it is on
/// and how far it is when the player is on that planet.
pub fn words(count: usize, picked: Option<(&str, Option<f64>)>) -> String {
    match (count, picked) {
        (0, _) => EMPTY.to_string(),
        (_, None) => PICK.to_string(),
        (_, Some((_, Some(away)))) => format!("Follow it by compass: {}.", distance_short(away)),
        (_, Some((planet, None))) => {
            let planet: String = planet.chars().take(10).collect();
            format!("Flies to {planet}, then follow.")
        }
    }
}

/// Returns how far `pin` is from the player at `here`, in metres, when they are on its planet.
fn away(pin: &Pin, here: Option<(BodyId, DVec3)>, universe: &Universe) -> Option<f64> {
    let (body, at) = here.filter(|(body, _)| *body == pin.body)?;
    let radius = universe.planet(body).radius;
    Some((DVec3::from_array(pin.dir) * radius - at).length())
}

/// What the pins are written from: the pins, the name typed, and where the player is.
type Kept<'w, 's> = (
    Res<'w, Places>,
    Res<'w, Typing>,
    Me<'w, 's>,
    Option<Res<'w, Universe>>,
);

/// Writes the rows of the pins, how far each is and the line under them.
fn pins_text((places, typing, me, universe): &Kept, faces: &mut Faces) {
    for (PinName(index), mut text) in &mut faces.names {
        if let Some(name) = shown(places, typing, *index) {
            text.set_if_neq(Text(name));
        }
    }
    let Some(universe) = universe else {
        return;
    };
    let here = me.spot();
    let far = |pin: &Pin| match away(pin, here, universe) {
        Some(metres) => distance_short(metres),
        None => universe.planet(pin.body).name,
    };
    for (PinFar(index), mut text) in &mut faces.far {
        if let Some(pin) = places.pins.get(*index) {
            text.set_if_neq(Text(far(pin)));
        }
    }
    let picked = places.selected.and_then(|index| places.pins.get(index));
    let picked = picked.map(|pin| (universe.planet(pin.body).name, away(pin, here, universe)));
    let said = words(
        places.pins.len(),
        picked
            .as_ref()
            .map(|(name, metres)| (name.as_str(), *metres)),
    );
    for mut text in &mut faces.hint {
        text.set_if_neq(Text(said.clone()));
    }
}

/// Writes the list of players, the markers at `friends` in `markers`: their names and their
/// colours.
fn players_text(markers: &[Marker], friends: &[usize], faces: &mut Faces) {
    let friend = |index: usize| friends.get(index).and_then(|at| markers.get(*at));
    for (PlayerName(index), mut text) in &mut faces.players {
        if let Some(friend) = friend(*index).filter(|friend| text.0 != friend.label) {
            text.0 = friend.label.clone();
        }
    }
    for (PlayerDot(index), mut fill) in &mut faces.dots {
        if let Some(friend) = friend(*index) {
            fill.set_if_neq(BackgroundColor(friend.color));
        }
    }
}

/// Writes both lists of the side panel and hides the rows past their ends or past the room of
/// the window, the buttons of the pin while none is selected, and the players outside a
/// session.
pub(super) fn write(
    (kept, markers, board): (Kept, Res<Markers>, Res<Board>),
    session: Option<Res<Link>>,
    (mut rows, mut faces): (Rows, Faces),
    mut selected: Query<(&PinRow, &mut Selected)>,
    mut friends: Local<Vec<usize>>,
) {
    friends.clear();
    let found = markers.0.iter().enumerate();
    let found = found.filter(|(_, marker)| marker.kind == MarkerKind::Friend);
    friends.extend(found.map(|(index, _)| index));
    let places = &kept.0;
    let room = board.lists.unwrap_or((usize::MAX, usize::MAX));
    let (saved, met) = (places.pins.len().min(room.0), friends.len().min(room.1));
    let (session, pinned) = (session.is_some(), places.selected.is_some());
    for (mut node, pin, player, (block, alone, actions)) in &mut rows {
        let on = match (pin, player) {
            (Some(PinRow(index)), _) => *index < saved,
            (_, Some(PlayerRow(index))) => *index < met,
            _ if block => session || met > 0,
            _ if alone => session && met == 0,
            _ => actions && pinned,
        };
        let display = [Display::None, Display::Flex][usize::from(on)];
        if node.display != display {
            node.display = display;
        }
    }
    for (PinRow(index), mut chosen) in &mut selected {
        chosen.set_if_neq(Selected(places.selected == Some(*index)));
    }
    pins_text(&kept, &mut faces);
    players_text(&markers.0, &friends, &mut faces);
}

/// Writes the name of the Go key of the settings beside Go on the buttons of the selected pin.
pub(super) fn go_key(
    settings: Option<Res<Persistent<Settings>>>,
    mut keys: Query<&mut Text, With<PinGoKey>>,
) {
    let Some(settings) = settings else {
        return;
    };
    let name = Keys::of(&settings.controls.keys).go;
    for mut text in &mut keys {
        text.set_if_neq(Text(name.clone()));
    }
}
