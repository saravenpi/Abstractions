//! Entering a body: the planet or moon the world streams, swapped in whenever the player reaches another
//! one, and never a world without ground for a save to resume on.

use std::ops::DerefMut;
use std::sync::Arc;

use bevy::prelude::*;

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
