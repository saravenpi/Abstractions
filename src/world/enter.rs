//! Entering a body: the planet or moon the world streams, swapped in whenever the player reaches another
//! one, and never a world without ground, a body that does not exist or a place no body can be at for a
//! save to resume on.

use std::ops::DerefMut;
use std::sync::Arc;

use bevy::prelude::*;

use super::footing::{aboard, grounded};
use super::globe::Face;
use super::{Globe, Planet, Terrain};
use crate::life::Life;
use crate::saves::ActiveWorld;
use crate::space::Location;
use crate::universe::{BodyId, PlanetSpec, Universe};

/// Returns the streamed planet named `body` with the ground of `spec` and its `life`, a planet or a moon
/// alike, and its terrain view on the north face.
pub(super) fn planet_of(body: BodyId, spec: &PlanetSpec, life: Option<Life>) -> (Planet, Terrain) {
    let globe = Arc::new(Globe::of(spec).with_life(life));
    globe.warm_drainage();
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let planet = Planet {
        body,
        globe,
        day_seconds: spec.day_seconds,
    };
    (planet, terrain)
}

/// Makes `body` the streamed planet: swaps the globe and the terrain view, and has every chunk
/// of the previous planet dropped. A moon is entered like a planet, from its own spec.
///
/// The caller still owns the `Anchor` and `Location`. The terrain starts on the north face and
/// follows the focus from the next frame on.
pub fn enter_planet(commands: &mut Commands, universe: &Universe, body: BodyId) -> Terrain {
    let (planet, terrain) = {
        let spec = universe.planet(body);
        let life = crate::life::of(&spec, &universe.system(body.star).star);
        planet_of(body, &spec, life)
    };
    commands.insert_resource(terrain.clone());
    commands.insert_resource(planet);
    terrain
}

/// Enters the planet named by `Location` whenever it is not the streamed one.
pub(super) fn follow_body(
    mut commands: Commands,
    location: Option<Res<Location>>,
    universe: Option<Res<Universe>>,
    planet: Res<Planet>,
) {
    if let (Some(location), Some(universe)) = (location, universe)
        && location.body != planet.body
    {
        enter_planet(&mut commands, &universe, location.body);
    }
}

/// Forgets where the save of `active` stands when `deck` says that is on a body without ground, which
/// no body can stand on: the world then begins as a new game does, at the home base, with a fresh
/// ship berthed on its pad. The saved ship is dropped too, since it was left beside that body; with
/// no ship in the save, the day clock that moves to its saved phase has nothing in space to strand.
/// A save it keeps is not touched, nor marked changed.
pub(super) fn ashore<A>(active: &mut A, deck: impl Fn(BodyId) -> bool)
where
    A: DerefMut<Target = ActiveWorld>,
{
    if active.0.save.place.is_some_and(|place| deck(place.body)) {
        let save = &mut active.0.save;
        save.place = None;
        save.position = None;
        save.ship = None;
    }
}

/// What the player is told when the saved place was put on the ground in the same direction.
pub const GROUNDED: &str = "Your saved place could not be used. You are on the ground below it.";

/// What the player is told when the saved place was dropped and the world began at the base.
pub const AT_BASE: &str = "Your saved place could not be used. You are back at the base.";

/// A line to tell the player once play begins: the body is not where the save left it. The loading
/// screen inserts it too when it had to move the body, and shows it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resettled(pub &'static str);

/// Puts the saved place of `active` at a height a body can be at on `globe`, the ground of its body, whose
/// air ends `air` metres up, and returns true when it had to move a body on foot. A body seated in the
/// ship is where the ship is: its place only says where the world loads, and is moved without a word. A
/// save it keeps is not touched, nor marked changed.
pub(super) fn settle<A>(active: &mut A, globe: &Globe, air: f64) -> bool
where
    A: DerefMut<Target = ActiveWorld>,
{
    let Some(place) = active.0.save.place else {
        return false;
    };
    let seated = active.0.save.seated();
    let sound = match seated {
        true => aboard(place, globe),
        false => grounded(place, globe, air),
    };
    if sound == place {
        return false;
    }
    active.0.save.place = Some(sound);
    !seated
}

/// Enters the body a world starts on and returns it with its terrain view on the north face: the body
/// the save of `active` names, or the home planet. A save on foot on a body without ground, or on a body
/// that does not exist or at a place that points nowhere, begins as a new game, see [`ashore`], and a
/// place at a height no body on foot can be at is put on the ground, see [`settle`]. Either leaves a
/// [`Resettled`] line. A save seated in the ship keeps its body, ground or not: the ship is over it.
pub(super) fn arrive<A>(
    commands: &mut Commands,
    universe: &Universe,
    mut active: Option<&mut A>,
) -> (BodyId, Terrain)
where
    A: DerefMut<Target = ActiveWorld>,
{
    let mut line = None;
    if let Some(active) = active.as_mut() {
        let (place, seated) = (active.0.save.place, active.0.save.seated());
        let gone = place.is_some_and(|place| !place.pointed() || !universe.has(place.body));
        let deck = |body| !seated && !Globe::of(&universe.planet(body)).has_ground();
        ashore(&mut **active, |body| gone || deck(body));
        line = gone.then_some(AT_BASE);
    }
    let home = || universe.home();
    let body = active
        .as_ref()
        .map_or_else(home, |active| active.0.save.body(universe));
    let north = enter_planet(commands, universe, body);
    let air = universe.planet(body).atmosphere_height;
    if active.is_some_and(|active| settle(active, north.globe(), air)) {
        line = Some(GROUNDED);
    }
    if let Some(line) = line {
        commands.insert_resource(Resettled(line));
    }
    (body, north)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{DQuat, DVec3};

    use crate::saves::{Place, WorldEntry, WorldSave};
    use crate::ship::Ship;
    use crate::space::Frame;

    /// Returns a world whose save stands on `body`, its ship flying low over that body.
    fn standing_on(body: BodyId) -> ActiveWorld {
        let place = Place {
            body,
            dir: [0.0, 1.0, 0.0],
            altitude: 3.0,
        };
        let ship = Ship::at_rest(Frame::Surface(body), DVec3::Y * 9.0e4, DQuat::IDENTITY);
        let save = WorldSave {
            place: Some(place),
            ship: Some(toml::Value::try_from(ship.state()).unwrap()),
            ..WorldSave::default()
        };
        ActiveWorld(WorldEntry {
            id: "w".into(),
            save,
        })
    }

    /// A save on a body without ground loses its place and its ship and begins at home as a new
    /// game, so the ship is a fresh one berthed at the base; any other save is kept.
    #[test]
    fn a_save_on_a_deck_resumes_at_home() {
        let giant = BodyId {
            index: 4,
            ..BodyId::default()
        };
        let mut stranded = standing_on(giant);
        ashore(&mut &mut stranded, |body| body == giant);
        let save = &stranded.0.save;
        assert_eq!((save.place, save.position, &save.ship), (None, None, &None));
        let mut kept = standing_on(BodyId::default());
        ashore(&mut &mut kept, |body| body == giant);
        assert_eq!(kept, standing_on(BodyId::default()));
    }
}
