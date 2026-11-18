//! The patch: the square of ground one image of the minimap covers.
//!
//! A patch is fixed to the planet and wider than the view, so walking and turning only move
//! and turn the image. A new patch is painted when the player nears the edge of the one shown
//! or when the range changes. It is laid out on a metric chart around its middle, where
//! distances and bearings are true.

use bevy::math::{DVec2, DVec3, Vec2};

use crate::world::globe::Tangent;

/// Texels along one side of the image.
pub const TEXELS: usize = 96;

/// Half width of a patch over the range it was painted for. The view turns inside it, and
/// its corners reach 1.42 ranges from the player.
pub const SPREAD: f32 = 2.0;

/// How far from the middle of its patch, as a share of the range, the player walks before a
/// new patch is painted. The view stays inside the old one until 0.58.
pub const DRIFT: f32 = 0.3;

/// A square of ground around one point of a planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Patch {
    /// The chart of the ground around the middle of the patch.
    pub chart: Tangent,
    /// The range in metres the patch was painted for.
    pub range: f32,
}

impl Patch {
    /// Returns the patch around the unit direction `dir` of a planet of `radius` metres, for
    /// a map of `range` metres, with north up.
    pub fn around(dir: DVec3, radius: f64, range: f32) -> Patch {
        let north = DVec3::Y - dir * dir.y;
        let east = match north.length_squared() > 1e-9 {
            true => north.cross(dir),
            false => DVec3::X,
        };
        let chart = Tangent::new(dir, east, radius, DVec2::ZERO);
        Patch { chart, range }
    }

    /// Returns the metres from the middle of the patch to the middle of each side.
    pub fn half(&self) -> f32 {
        self.range * SPREAD
    }

    /// Returns the unit direction of the middle of the texel in column `x` and row `y`. Row 0
    /// is the north side.
    pub fn dir(&self, x: usize, y: usize) -> DVec3 {
        let along = |cell: usize| (cell as f64 + 0.5) / TEXELS as f64 - 0.5;
        let side = f64::from(self.half()) * 2.0;
        self.chart.dir(DVec2::new(along(x), along(y)) * side)
    }

    /// Returns how many metres east and north of the middle of the patch the unit direction
    /// `dir` lies.
    pub fn locate(&self, dir: DVec3) -> Vec2 {
        let at = self.chart.locate(dir);
        Vec2::new(at.x as f32, -at.y as f32)
    }

    /// Returns true when a new patch is due for a player `at` metres east and north of the
    /// middle and a map of `range` metres.
    pub fn stale(&self, at: Vec2, range: f32) -> bool {
        self.range != range || at.abs().max_element() > DRIFT * self.range
    }

    /// Returns the patch moved to the player `at` metres east and north of the middle, by a
    /// whole number of texels and without turning, so the patterns of the map stay still.
    pub fn moved(&self, at: Vec2) -> Patch {
        let texel = self.half() * 2.0 / TEXELS as f32;
        let whole = (at / texel).round() * texel;
        let middle = self
            .chart
            .dir(DVec2::new(f64::from(whole.x), -f64::from(whole.y)));
        let chart = Tangent::new(middle, self.chart.east, self.chart.radius, DVec2::ZERO);
        Patch { chart, ..*self }
    }
}

/// Returns the patch to paint for a player in the unit direction `dir` of a planet of `radius`
/// metres and a map of `range` metres, or nothing while the patch `shown` still serves.
pub fn next(shown: Option<Patch>, dir: DVec3, radius: f64, range: f32) -> Option<Patch> {
    let Some(patch) = shown.filter(|patch| patch.range == range) else {
        return Some(Patch::around(dir, radius, range));
    };
    let at = patch.locate(dir);
    patch.stale(at, range).then(|| patch.moved(at))
}
