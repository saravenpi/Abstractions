//! Shells: the stars of a galaxy in rings of distance around a point, found by visiting only the
//! sectors of the ring, never the whole cube around it.

use std::f64::consts::PI;

use bevy::math::{DVec3, IVec3};

use super::galaxy::Shape;
use super::ids::GalaxyId;
use super::spec::StarSpec;
use super::{LY, SECTOR_LY, sector};

/// Share of the radius of a galaxy at which the Sun lies from the centre: 26,000 of 50,000
/// light-years in the Milky Way, which is 25,000 to 28,000 from it
/// (<https://en.wikipedia.org/wiki/Milky_Way>).
const SUN_RADIUS_SHARE: f64 = 0.52;

/// Returns the Sun-like point of a galaxy, in metres from its centre: 0.52 radii out in the
/// plane of the disc, at the angle between two arms where the wave of the arms is at its
/// lowest, as the Sun lies between the Sagittarius and Perseus arms.
pub(super) fn sun_point(shape: &Shape) -> DVec3 {
    let out = SUN_RADIUS_SHARE;
    let between = PI / f64::from(shape.arms.max(1));
    let angle = shape.swirl * out + between;
    DVec3::new(angle.cos(), 0.0, angle.sin()) * out * shape.radius * LY
}

/// Returns the rows of sectors along Y whose centres lie from `lo` to `hi` light-years above or
/// below the height `from_y` in light-years, the ones above first.
///
/// The ranges are a little wide: the caller keeps the sectors that really are in the shell.
fn rows(from_y: f64, (lo, hi): (f64, f64)) -> impl Iterator<Item = i32> {
    let row = |y: f64| (y / SECTOR_LY - 0.5).floor() as i32;
    let above = row(from_y + lo)..=row(from_y + hi) + 1;
    let below = row(from_y - hi)..=(row(from_y - lo) + 1).min(row(from_y + lo) - 1);
    above.chain(below)
}

/// A shell of a galaxy: the stars from `low` to `high` light-years of one point.
struct Ring<'a> {
    /// The galaxy the sectors belong to.
    shape: &'a Shape,
    /// Identifier of that galaxy.
    galaxy: GalaxyId,
    /// The point in light-years from the galaxy centre.
    at: DVec3,
    /// Inner distance of the shell in light-years.
    low: f64,
    /// Outer distance of the shell in light-years.
    high: f64,
}

impl Ring<'_> {
    /// Appends the stars of the sectors of column (`x`, `z`) whose centre lies in the shell.
    fn column(&self, x: i32, z: i32, out: &mut Vec<StarSpec>) {
        let side = |i: i32, c: f64| (f64::from(i) + 0.5) * SECTOR_LY - c;
        let flat = side(x, self.at.x).powi(2) + side(z, self.at.z).powi(2);
        if flat >= self.high * self.high {
            return;
        }
        let near = (self.low * self.low - flat).max(0.0).sqrt();
        let far = (self.high * self.high - flat).sqrt();
        for y in rows(self.at.y, (near, far)) {
            let sector = IVec3::new(x, y, z);
            let out_ly = ((sector.as_dvec3() + 0.5) * SECTOR_LY - self.at).length();
            if (self.low..self.high).contains(&out_ly) {
                sector::fill(self.shape, self.galaxy, sector, out);
            }
        }
    }
}

/// Returns the stars of `galaxy` whose sector centre lies from `low` to `low + width`
/// light-years from `from`, a point in metres from the galaxy centre, nearest first.
///
/// For each column of sectors it works out the rows the shell crosses, so a shell costs about
/// the number of sectors it holds, and a search that grows shell by shell never reads the empty
/// inside again.
pub(super) fn shell(
    shape: &Shape,
    galaxy: GalaxyId,
    from: DVec3,
    (low, width): (f64, f64),
) -> Vec<StarSpec> {
    let at = from / LY;
    let ring = Ring {
        shape,
        galaxy,
        at,
        low,
        high: low + width,
    };
    let reach = (ring.high / SECTOR_LY).ceil() as i32 + 1;
    let base = (at / SECTOR_LY).floor().as_ivec3();
    let mut stars = Vec::new();
    for x in base.x - reach..=base.x + reach {
        for z in base.z - reach..=base.z + reach {
            ring.column(x, z, &mut stars);
        }
    }
    let by = |s: &StarSpec| s.pos.distance_squared(from);
    stars.sort_by(|a, b| by(a).total_cmp(&by(b)));
    stars
}
