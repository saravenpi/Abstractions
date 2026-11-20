//! Sectors: the stars of one 10 gly cube of a galaxy, and the lookups built on them.

use bevy::math::{DVec3, IVec3};

use super::galaxy::{self, Shape};
use super::hash::{self, Rng};
use super::ids::{GalaxyId, StarId};
use super::spec::StarSpec;
use super::{GLY, SECTOR_GLY, Universe, star};
use crate::space::UniPos;

/// Most stars one sector holds.
pub const SECTOR_STARS: u8 = 12;

/// Appends the stars of one sector to `out`, in index order.
pub(super) fn fill(shape: &Shape, galaxy: GalaxyId, sector: IVec3, out: &mut Vec<StarSpec>) {
    let parts = shape.parts((sector.as_dvec3() + 0.5) * SECTOR_GLY);
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
    (pos / (SECTOR_GLY * GLY)).floor().as_ivec3()
}

impl Universe {
    /// Returns one star, or nothing when its galaxy, sector or index does not exist.
    pub fn star(&self, id: StarId) -> Option<StarSpec> {
        let mut stars = self.stars_in_sector(id.galaxy, id.sector);
        (usize::from(id.index) < stars.len()).then(|| stars.swap_remove(usize::from(id.index)))
    }

    /// Returns the stars within `radius_gly` game light-years of `pos`, nearest first.
    ///
    /// `pos` is in metres from the centre of `galaxy`, like [`StarSpec::pos`]. The cost grows with
    /// the cube of the radius: 60 gly reads about 2,200 sectors.
    pub fn stars_near(&self, galaxy: GalaxyId, pos: DVec3, radius_gly: f64) -> Vec<StarSpec> {
        let mut stars = Vec::new();
        let Some(shape) = galaxy::shape(self.seed, galaxy) else {
            return stars;
        };
        let reach = radius_gly.max(0.0) * GLY;
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
        let sector = self.galaxy(id.galaxy)?.center_sector + id.sector;
        Some(UniPos::sectors(id.galaxy.0, sector, star.local))
    }
}

impl super::GalaxySpec {
    /// Returns the universe position of a point `pos` metres from the centre of this galaxy.
    ///
    /// The centre is a whole sector, so the result is exact up to the 64-bit step of `pos`.
    pub fn locate(&self, pos: DVec3) -> UniPos {
        UniPos::sectors(self.id.0, self.center_sector, DVec3::ZERO) + pos
    }
}
