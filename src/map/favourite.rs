//! Favourites: places the player saved with a name. They live in the world save, are listed
//! in the side panel of the map and drawn on it as hollow squares.
//!
//! A new favourite is named at once: the letters typed go into its name, Enter keeps it and
//! Escape keeps the name it had. Renaming one works the same way.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::computer::Typing;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::saves::{ActiveWorld, Favourite};
use crate::universe::{BodyId, Universe};

/// Characters a name keeps at most.
pub const NAME_CHARS: usize = 32;

/// The favourites of the player, in the order they were saved.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Favourites(pub Vec<Favourite>);

impl Favourites {
    /// Saves the place `dir` of `body` under a name of its own and returns its index.
    pub fn save(&mut self, body: BodyId, dir: DVec3) -> usize {
        let name = format!("Place {}", self.0.len() + 1);
        let dir = dir.normalize_or(DVec3::Y).to_array();
        self.0.push(Favourite { name, body, dir });
        self.0.len() - 1
    }

    /// Gives the favourite at `index` the name `name`, unless the name is blank.
    pub fn rename(&mut self, index: usize, name: &str) {
        let name = name.trim();
        if let Some(favourite) = self.0.get_mut(index).filter(|_| !name.is_empty()) {
            favourite.name = name.chars().take(NAME_CHARS).collect();
        }
    }
}

/// Applies one pressed key to a name and returns true when the name is done: Enter keeps it,
/// Escape gives it back as it was, and `before` is that name.
pub fn edit(name: &mut String, key: &Key, before: &str) -> bool {
    match key {
        Key::Enter => true,
        Key::Escape => {
            *name = before.to_string();
            true
        }
        Key::Backspace => {
            name.pop();
            false
        }
        Key::Character(text) if name.chars().count() < NAME_CHARS => {
            name.extend(text.chars().filter(|c| !c.is_control()));
            false
        }
        _ => false,
    }
}

/// Types into the name of the favourite being named, and stores it when it is done.
pub(super) fn name_keys(
    mut typed: MessageReader<KeyboardInput>,
    mut typing: ResMut<Typing>,
    mut favourites: ResMut<Favourites>,
) {
    let Typing::Name(index, name) = &mut *typing else {
        typed.clear();
        return;
    };
    let index = *index;
    let before = favourites
        .0
        .get(index)
        .map_or(String::new(), |fav| fav.name.clone());
    let mut pressed = typed.read().filter(|key| key.state == ButtonState::Pressed);
    let done = pressed.any(|key| edit(name, &key.logical_key, &before));
    if done {
        let name = name.clone();
        favourites.rename(index, &name);
        *typing = Typing::Off;
    }
}

/// Reads the favourites of the world that was just made active.
pub(super) fn load(active: Option<Res<ActiveWorld>>, mut favourites: ResMut<Favourites>) {
    let Some(active) = active.filter(|active| active.is_added()) else {
        return;
    };
    *favourites = Favourites(active.0.save.marks.clone());
}

/// Writes the favourites into the active world whenever they change, so the next save keeps them.
pub(super) fn store(favourites: Res<Favourites>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| favourites.is_changed()) else {
        return;
    };
    if active.0.save.marks != favourites.0 {
        active.0.save.marks = favourites.0.clone();
    }
}

/// Writes every favourite as a marker when they change.
pub(super) fn fill(
    favourites: Res<Favourites>,
    universe: Option<Res<Universe>>,
    mut markers: ResMut<Markers>,
) {
    let Some(universe) = universe.filter(|_| favourites.is_changed()) else {
        return;
    };
    markers.clear(MarkerKind::Favourite, 0);
    for favourite in &favourites.0 {
        let radius = universe.planet(favourite.body).radius;
        markers.0.push(Marker {
            kind: MarkerKind::Favourite,
            place: MarkerPlace::Body(favourite.body, DVec3::from_array(favourite.dir) * radius),
            color: Color::WHITE,
            label: favourite.name.clone(),
            owner: 0,
        });
    }
}
