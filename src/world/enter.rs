//! Entering a body: the planet or moon the world streams, swapped in whenever the player reaches another
//! one, and never a world without ground for a save to resume on.

use std::ops::DerefMut;
use std::sync::Arc;

use bevy::prelude::*;

use super::globe::Face;
use super::{Globe, Planet, Terrain};
use crate::saves::ActiveWorld;
use crate::space::Location;
use crate::universe::{BodyId, PlanetSpec, Universe};

/// Returns the streamed planet named `body` with the ground of `spec`, a planet or a moon alike, and its
/// terrain view on the north face.
pub(super) fn planet_of(body: BodyId, spec: &PlanetSpec) -> (Planet, Terrain) {
    let globe = Arc::new(Globe::of(spec));
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
    let (planet, terrain) = planet_of(body, &universe.planet(body));
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
/// no body can stand on: the world then begins as a new game does, at the home base. A save it keeps
/// is not touched, nor marked changed.
pub(super) fn ashore<A>(active: &mut A, deck: impl Fn(BodyId) -> bool)
where
    A: DerefMut<Target = ActiveWorld>,
{
    if active.0.save.place.is_some_and(|place| deck(place.body)) {
        let save = &mut active.0.save;
        save.place = None;
        save.position = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saves::{Place, WorldEntry, WorldSave};

    /// Returns a world whose save stands on `body`.
    fn standing_on(body: BodyId) -> ActiveWorld {
        let place = Place {
            body,
            dir: [0.0, 1.0, 0.0],
            altitude: 3.0,
        };
        let save = WorldSave {
            place: Some(place),
            ..WorldSave::default()
        };
        ActiveWorld(WorldEntry {
            id: "w".into(),
            save,
        })
    }

    /// A save on a body without ground loses its place and begins at home; any other save is kept.
    #[test]
    fn a_save_on_a_deck_resumes_at_home() {
        let giant = BodyId {
            index: 4,
            ..BodyId::default()
        };
        let mut stranded = standing_on(giant);
        ashore(&mut &mut stranded, |body| body == giant);
        assert_eq!(
            (stranded.0.save.place, stranded.0.save.position),
            (None, None)
        );
        let mut kept = standing_on(BodyId::default());
        ashore(&mut &mut kept, |body| body == giant);
        assert_eq!(kept, standing_on(BodyId::default()));
    }
}
