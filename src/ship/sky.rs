//! What surrounds the ship: the planet under it, the bodies of its system and the stars nearby.

use std::sync::Arc;

use bevy::math::{DVec3, IVec3};
use bevy::prelude::*;

use super::body::Ship;
use super::wells::{Well, system_wells};
use crate::space::Frame;
use crate::universe::{
    BodyId, GalaxyId, PlanetSpec, StarId, StarSpec, SystemSpec, Universe, sector_of,
};
use crate::world::{Globe, Planet};

/// Game light-years around the ship in which stars are looked up between the stars.
const STAR_SCAN: f64 = 12.0;

/// The planet whose surface frame the ship is in.
struct Land {
    /// Which planet it is.
    body: BodyId,
    /// What the universe says about it.
    spec: PlanetSpec,
    /// Its ground.
    globe: Arc<Globe>,
}

/// The surroundings of the ship, looked up once and kept while they stay the same.
#[derive(Resource, Default)]
pub struct Skies {
    /// The system the ship is in, with the star that was asked for.
    system: Option<(StarId, SystemSpec)>,
    /// The planet whose surface frame the ship is in.
    land: Option<Land>,
    /// Stars around the ship between the stars, and the galaxy sector they were looked up from.
    stars: Option<(GalaxyId, IVec3, Vec<StarSpec>)>,
    /// Bodies around the ship at the start of the frame, in the frame of the ship away from a
    /// surface and in the frame of the star on a surface.
    pub wells: Vec<Well>,
    /// The destination of a long jump in the frame of the ship. While it is set, the stars on
    /// the way are ignored between the stars.
    pub beacon: Option<Well>,
}

impl Skies {
    /// Returns the system of `star`, looked up when it is not the one kept.
    pub fn system(&mut self, universe: &Universe, star: StarId) -> &SystemSpec {
        let kept = self.system.take().filter(|(id, _)| *id == star);
        &self
            .system
            .insert(kept.unwrap_or_else(|| (star, universe.system(star))))
            .1
    }

    /// Returns the stars within a few light-years of `pos` in `galaxy`, looked up again when the
    /// ship changes sector.
    pub fn stars(&mut self, universe: &Universe, galaxy: GalaxyId, pos: DVec3) -> &[StarSpec] {
        let sector = sector_of(pos);
        let kept = self
            .stars
            .take()
            .filter(|(g, s, _)| (*g, *s) == (galaxy, sector));
        let fresh = || (galaxy, sector, universe.stars_near(galaxy, pos, STAR_SCAN));
        &self.stars.insert(kept.unwrap_or_else(fresh)).2
    }

    /// Keeps the planet `body` as the land under the ship, with the globe of the streamed planet
    /// when it is the same one.
    fn land(&mut self, universe: &Universe, streamed: Option<&Planet>, body: BodyId) {
        if self.land.as_ref().is_some_and(|land| land.body == body) {
            return;
        }
        let spec = universe.planet(body);
        let shared = streamed.filter(|planet| planet.body == body);
        let globe =
            shared.map_or_else(|| Arc::new(Globe::of(&spec)), |planet| planet.globe.clone());
        self.land = Some(Land { body, spec, globe });
    }

    /// Returns the system kept, the one the ship was last in.
    pub fn spec(&self) -> Option<&SystemSpec> {
        self.system.as_ref().map(|(_, system)| system)
    }

    /// Returns the planet under the ship, when the ship has been in a surface frame.
    pub fn planet(&self) -> Option<&PlanetSpec> {
        self.land.as_ref().map(|land| &land.spec)
    }

    /// Returns the terrain height in metres above sea level at a position in the frame of the
    /// planet under the ship, zero without one.
    pub fn height(&self, pos: DVec3) -> f64 {
        let Some(dir) = pos.try_normalize() else {
            return 0.0;
        };
        self.land
            .as_ref()
            .map_or(0.0, |land| f64::from(land.globe.height(dir)))
    }

    /// Looks up what surrounds `ship` at `time` seconds and samples the wells.
    pub fn refresh(
        &mut self,
        universe: &Universe,
        streamed: Option<&Planet>,
        ship: &Ship,
        time: f64,
    ) {
        self.wells = match (ship.frame, self.beacon) {
            (Frame::Surface(body), _) => {
                self.land(universe, streamed, body);
                system_wells(self.system(universe, body.star), time)
            }
            (Frame::System(star), _) => system_wells(self.system(universe, star), time),
            (Frame::Galaxy(_), Some(beacon)) => vec![beacon],
            (Frame::Galaxy(galaxy), None) => {
                let stars = self.stars(universe, galaxy, ship.pos);
                stars
                    .iter()
                    .map(|star| Well::star(star, star.pos))
                    .collect()
            }
        };
    }
}
