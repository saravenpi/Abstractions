//! Sectors: the stars of one 10 light-year cube of a galaxy, and the lookups built on them,
//! down to whether a planet or a moon exists.

use bevy::math::{DVec3, IVec3};

use super::galaxy::{self, Shape};
use super::hash::{self, Rng};
use super::ids::{BodyId, GalaxyId, StarId};
use super::spec::StarSpec;
use super::{LY, SECTOR_LY, SECTOR_M, Universe, moon, star};
use crate::space::UniPos;

/// Most stars one sector holds.
pub const SECTOR_STARS: u8 = 12;

/// Appends the stars of one sector to `out`, in index order.
pub(super) fn fill(shape: &Shape, galaxy: GalaxyId, sector: IVec3, out: &mut Vec<StarSpec>) {
    let parts = shape.parts((sector.as_dvec3() + 0.5) * SECTOR_LY);
    let expected = parts.total();
    if expected <= 0.0 {
        return;
    }
    let mut rng = Rng::new(hash::cell(shape.seed, sector));
    let extra = u8::from(rng.unit() < expected.fract());
    let count = (expected as u8).saturating_add(extra).min(SECTOR_STARS);
    for index in 0..count {
        let id = StarId {
            galaxy,
            sector,
            index,
        };
        out.push(star::build(id, rng.next(), parts.region()));
    }
}

/// Returns the sector that holds a point `pos` metres from a galaxy centre.
pub fn sector_of(pos: DVec3) -> IVec3 {
    (pos / SECTOR_M).floor().as_ivec3()
}

impl Universe {
    /// Returns true when the planet or moon `id` exists: its star does, the star has a planet of
    /// that rank, and the planet a moon of that rank. [`Universe::planet`] answers for any id,
    /// so ask this first for an id read from a file or sent by a server.
    pub fn has(&self, id: BodyId) -> bool {
        let Some(star) = self.star(id.star) else {
            return false;
        };
        let planets = self.planets_of(&star);
        let planet = planets.get(usize::from(id.index));
        planet.is_some_and(|planet| id.moon == 0 || moon::ranked(&star, planet, id.moon).is_some())
    }

    /// Returns one star, or nothing when its galaxy, sector or index does not exist.
    pub fn star(&self, id: StarId) -> Option<StarSpec> {
        let mut stars = self.stars_in_sector(id.galaxy, id.sector);
        (usize::from(id.index) < stars.len()).then(|| stars.swap_remove(usize::from(id.index)))
    }

    /// Returns the stars within `radius_ly` light-years of `pos`, nearest first.
    ///
    /// `pos` is in metres from the centre of `galaxy`, like [`StarSpec::pos`]. The cost grows with
    /// the cube of the radius: 55 light-years read about 2,200 sectors. Ask for no more than 60.
    pub fn stars_near(&self, galaxy: GalaxyId, pos: DVec3, radius_ly: f64) -> Vec<StarSpec> {
        let mut stars = Vec::new();
        let Some(shape) = galaxy::shape(self.seed, galaxy) else {
            return stars;
        };
        let reach = radius_ly.max(0.0) * LY;
        let (low, high) = (sector_of(pos - reach), sector_of(pos + reach));
        let n = high - low + 1;
        for i in 0..n.x * n.y * n.z {
            let sector = low + IVec3::new(i % n.x, i / n.x % n.y, i / (n.x * n.y));
            fill(&shape, galaxy, sector, &mut stars);
        }
        stars.retain(|s| s.pos.distance_squared(pos) <= reach * reach);
        stars.sort_by(|a, b| {
            a.pos
                .distance_squared(pos)
                .total_cmp(&b.pos.distance_squared(pos))
        });
        stars
    }

    /// Returns where a star is in the universe, or nothing when the star does not exist.
    pub fn star_position(&self, id: StarId) -> Option<UniPos> {
        let star = self.star(id)?;
        let galaxy = self.galaxy(id.galaxy)?;
        Some(UniPos::sectors(
            id.galaxy.0,
            galaxy.center_sector + id.sector,
            star.local,
        ))
    }
}

impl super::GalaxySpec {
    /// Returns the universe position of a point `pos` metres from the centre of this galaxy.
    ///
    /// The position is built from whole sectors and the metres inside the sector, so the 64-bit
    /// rounding of a place far from the centre stays out of it.
    pub fn locate(&self, pos: DVec3) -> UniPos {
        let sector = sector_of(pos);
        let local = pos - sector.as_dvec3() * SECTOR_M;
        UniPos::sectors(self.id.0, self.center_sector + sector, local)
    }
}
