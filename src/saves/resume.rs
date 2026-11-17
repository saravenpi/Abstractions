//! Resuming a world: bringing an old file up to date and finding where the player starts.

use bevy::prelude::*;

use super::model::{SAVE_VERSION, WorldSave};
use crate::universe::{BodyId, Universe};
use crate::world::Terrain;
use crate::world::globe::Chart;

impl WorldSave {
    /// Brings a file from an older version up to date.
    ///
    /// A file from before v0.6 holds a position on the flat world, which means nothing on a
    /// globe: it is dropped, so the world starts at the home planet spawn. The name, the seed,
    /// the play time and the creation date are kept.
    pub fn upgrade(&mut self) {
        if self.version < SAVE_VERSION {
            self.position = None;
            self.version = SAVE_VERSION;
        }
    }

    /// Returns the planet the world starts on: the one the player last stood on, or the home planet.
    pub fn body(&self, universe: &Universe) -> BodyId {
        self.place
            .map_or_else(|| universe.home(), |place| place.body)
    }

    /// Returns the chart position where the player starts on `terrain`: the saved spot, or the
    /// terrain spawn when there is none.
    ///
    /// `terrain` must be a view of the planet given by [`WorldSave::body`] whose chart covers the
    /// saved spot. The legacy flat terrain reads the flat position instead.
    pub fn spawn(&self, terrain: &Terrain) -> Vec3 {
        let radius = terrain.globe().radius();
        let saved = match terrain.chart() {
            Chart::Flat => self.position.map(Vec3::from),
            _ => self
                .place
                .and_then(|place| place.position(radius))
                .map(|at| terrain.locate(at)),
        };
        saved.unwrap_or_else(|| terrain.find_spawn())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::saves::capture::stand;
    use crate::space::Anchor;
    use crate::world::Globe;
    use crate::world::globe::Face;

    /// A spot captured on a globe resolves to the same chart position, and the flat position goes.
    #[test]
    fn a_captured_place_resolves_to_the_same_spot() {
        let globe = Arc::new(Globe::new(4, 6000.0, 22.0));
        let body = Universe::new(4).home();
        for face in Face::ALL {
            let terrain = Terrain::on(globe.clone(), globe.face_chart(face)).around(900.0, -350.0);
            let at = Vec3::new(905.0, 7.5, -341.0);
            let anchor = Anchor::surface(body, &terrain, Vec3::new(900.0, 7.5, -350.0));
            let mut save = WorldSave {
                position: Some([1.0, 2.0, 3.0]),
                ..WorldSave::default()
            };
            stand(&mut save, Some(&anchor), globe.radius(), at);
            assert_eq!(save.position, None);
            assert_eq!(save.body(&Universe::new(4)), body);
            let place = save.place.unwrap();
            assert!(
                (place.altitude - 7.5).abs() < 0.02,
                "{face:?} {}",
                place.altitude
            );
            assert!((save.spawn(&terrain) - at).length() < 0.02, "{face:?}");
        }
    }

    /// On the flat world the flat position is kept, and a place without a direction is ignored.
    #[test]
    fn the_flat_world_keeps_its_position() {
        let terrain = Terrain::new(4);
        let anchor = Anchor::surface(Universe::new(4).home(), &terrain, Vec3::ZERO);
        let mut save = WorldSave::default();
        stand(&mut save, Some(&anchor), 6000.0, Vec3::new(5.0, 6.0, 7.0));
        assert_eq!((save.position, save.place), (Some([5.0, 6.0, 7.0]), None));
        assert_eq!(save.spawn(&terrain), Vec3::new(5.0, 6.0, 7.0));
        save.place = Some(Default::default());
        assert_eq!(save.place.unwrap().position(6000.0), None);
    }
}
