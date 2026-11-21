//! What surrounds the ship: the planet or moon under it, the bodies of its system and the stars
//! nearby.

use std::sync::Arc;

use bevy::math::{DVec3, IVec3};
use bevy::prelude::*;

mod arrive;

use super::body::Ship;
use super::moons::with_moons;
use super::wells::{Well, system_wells};
use crate::home::Base;
use crate::space::Frame;
use crate::universe::{
    BodyId, GalaxyId, PlanetSpec, StarId, StarSpec, SystemSpec, Universe, sector_of,
};
use crate::world::{Globe, Planet};

/// Light-years around the ship in which stars are looked up between the stars. A sector is
/// `SECTOR_LY` wide, so 12 light-years read 64 to 125 sectors, about 29 stars at the density of
/// the Sun, once per sector the ship crosses.
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
    /// The system the ship is in, with the star that was asked for. Its `planets` list the
    /// moons of the planets after the planets, so the moons are bodies like any other.
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
    /// The body a course is aimed at and the radius of its lock while the course flies to it: its
    /// arrival shell, which the drive may reach, where every other lock sphere stays as it is.
    pub arrive: Option<(BodyId, f64)>,
    /// The home base, whose landing pad is ground the ship can stand on.
    pub base: Option<Base>,
}

impl Skies {
    /// Returns the system of `star` with the moons after the planets, looked up when it is not
    /// the one kept.
    pub fn system(&mut self, universe: &Universe, star: StarId) -> &SystemSpec {
        let kept = self.system.take().filter(|(id, _)| *id == star);
        let fresh = || (star, with_moons(universe, universe.system(star)));
        &self.system.insert(kept.unwrap_or_else(fresh)).1
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

    /// Keeps the planet or moon `body` as the land under the ship, with the globe of the streamed
    /// planet when it is the same one.
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

    /// Returns the height in metres above sea level of what the ship can stand on at a position
    /// in the frame of the planet under it: the terrain, or the landing pad of the home base
    /// where it is higher. Zero without a planet.
    pub fn height(&self, pos: DVec3) -> f64 {
        let (Some(dir), Some(land)) = (pos.try_normalize(), self.land.as_ref()) else {
            return 0.0;
        };
        let ground = f64::from(land.globe.height(dir));
        let on_pad = |base: &Base| base.body == land.body && base.on_pad(pos);
        let pad = self.base.filter(on_pad);
        pad.map_or(ground, |base| {
            ground.max(base.pad.length() - land.spec.radius)
        })
    }

    /// Returns the height in metres of the liquid surface of the planet under the ship: zero with
    /// a sea, lower than any ground on a dry planet, and zero without a planet.
    pub fn sea(&self) -> f64 {
        self.land
            .as_ref()
            .map_or(0.0, |land| f64::from(land.globe.sea_level()))
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
        self.shrink_target();
    }
}
