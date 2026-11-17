//! How many stars a place in a galaxy holds: core, disc, swirled arms, halo and a void at the centre.

use bevy::math::DVec3;

use super::GLY;
use super::galaxy::Shape;
use super::hash;
use super::home::HOME_GALAXY;
use super::spec::{GalaxyKind, GalaxySpec};

/// Stars per sector at the peak of the core.
const CORE: f64 = 3.0;

/// Stars per sector at the peak of the plain disc.
const DISC: f64 = 0.5;

/// Stars per sector at the peak of an arm or a clump.
const ARM: f64 = 2.0;

/// Stars per sector at the peak of a ball galaxy.
const BALL: f64 = 2.5;

/// Stars per sector at the peak of the halo.
const HALO: f64 = 0.004;

/// The part of a galaxy a place belongs to, which decides how old its stars are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Region {
    /// The dense centre, old stars.
    Core,
    /// An arm or a clump, where young hot stars form.
    Arm,
    /// The disc between the arms.
    Disc,
    /// The thin shell around the galaxy, old stars.
    Halo,
}

/// Expected stars per sector at one place, split by the structure they belong to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Parts {
    /// Stars of the dense core.
    pub core: f64,
    /// Stars of the arms or clumps.
    pub arm: f64,
    /// Stars of the plain disc or ball.
    pub disc: f64,
    /// Stars of the halo.
    pub halo: f64,
}

impl Parts {
    /// Returns the expected number of stars per sector.
    pub fn total(&self) -> f64 {
        self.core + self.arm + self.disc + self.halo
    }

    /// Returns the structure that holds most of the stars here.
    pub fn region(&self) -> Region {
        let ranked = [
            (self.disc, Region::Disc),
            (self.arm, Region::Arm),
            (self.core, Region::Core),
            (self.halo, Region::Halo),
        ];
        let best = |a: (f64, Region), b: (f64, Region)| if b.0 > a.0 { b } else { a };
        ranked.into_iter().fold(ranked[0], best).1
    }
}

/// Returns 0 below `a`, 1 above `b` and a smooth blend between (also works when `a > b`).
fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Shape {
    /// Returns the expected stars per sector at `p` game light-years from the galaxy centre.
    pub fn parts(&self, p: DVec3) -> Parts {
        let q = p / self.radius;
        let (r, d) = (q.x.hypot(q.z), q.length());
        let alive = smooth(0.01, 0.03, d) * smooth(1.4, 1.0, d);
        let (disc, arm) = match self.kind {
            GalaxyKind::Spiral => self.spiral(q, r),
            GalaxyKind::Elliptical => (self.ball(q, r), 0.0),
            GalaxyKind::Irregular => self.cloud(q, r),
        };
        Parts {
            core: alive * CORE * (-0.5 * (d / self.core).powi(2)).exp(),
            arm: alive * arm,
            disc: alive * disc,
            halo: alive * HALO * (-d / 0.4).exp(),
        }
    }

    /// Returns the disc and arm densities of a spiral at `q` radii from the centre, `r` from the axis.
    ///
    /// The arms are straight spokes turned by an angle that grows with the distance, which swirls them.
    fn spiral(&self, q: DVec3, r: f64) -> (f64, f64) {
        let plane = (-0.5 * (q.y / self.thick).powi(2) - r / 0.3).exp() * smooth(1.0, 0.9, r);
        let spoke = q.z.atan2(q.x) - self.swirl * r;
        let wave = 0.5 + 0.5 * (f64::from(self.arms) * spoke).cos();
        (DISC * plane, ARM * plane * wave.powi(3))
    }

    /// Returns the density of a featureless, slightly flattened ball of stars.
    fn ball(&self, q: DVec3, r: f64) -> f64 {
        let e = r.hypot(q.y / 0.7);
        BALL * (-e / 0.18).exp() * smooth(1.0, 0.8, e)
    }

    /// Returns the thin and clumped densities of an irregular cloud, the clumps hashed on a coarse grid.
    fn cloud(&self, q: DVec3, r: f64) -> (f64, f64) {
        let plane =
            (-0.5 * (q.y / (4.0 * self.thick)).powi(2) - r / 0.35).exp() * smooth(1.0, 0.8, r);
        let clump = hash::cell(self.seed, (q * 12.0).floor().as_ivec3()) as f64 / u64::MAX as f64;
        (0.3 * DISC * plane, ARM * plane * clump * clump)
    }
}

impl GalaxySpec {
    /// Returns the expected number of stars per sector at `pos` metres from the galaxy centre.
    ///
    /// Cheap enough to sample a few thousand times to draw an impression of the galaxy. The disc
    /// lies on the XZ plane. Values run from 0 in the void to about 3 in the core.
    pub fn density(&self, pos: DVec3) -> f64 {
        Shape::new(self.seed, self.id == HOME_GALAXY)
            .parts(pos / GLY)
            .total()
    }
}
