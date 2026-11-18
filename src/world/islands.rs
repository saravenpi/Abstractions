//! Land shape: warped continent mask with bays, islets, sandbars, cliffs and the elevation curves.

mod field;
mod fractal;
mod point;
mod point2;
mod point3;
mod relief;
mod seabed;
mod solid;

#[cfg(test)]
mod tests;

use noise::{Fbm, MultiFractal, Perlin};

pub use point::{Point, plus};
pub(in crate::world) use solid::gradient;

use super::biome::Climate;
use relief::Relief;
use seabed::Seabed;

/// Continent noise value above which the ground rises out of the sea.
const LAND_LEVEL: f32 = 0.16;

/// Sea-side inland measure below which the cliff weight no longer changes the sea floor.
const CLIFF_REACH: f32 = -0.38;

/// Where the large warp is read for each axis of the displacement, in noise units.
const WARP_SHIFT: [[f64; 3]; 3] = [[0.0; 3], [31.7, -17.3, 12.4], [-23.9, 8.6, -41.2]];

/// Where the small coast warp is read for each axis of the displacement, in noise units.
const COAST_SHIFT: [[f64; 3]; 3] = [[0.0; 3], [9.1, -4.3, 6.7], [-5.2, 2.8, -7.9]];

/// Returns 0 below `a`, 1 above `b` and a smooth blend between (also works when `a > b`).
pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Derives the noise seed number `k` from the world seed.
pub fn sub_seed(seed: u64, k: u32) -> u32 {
    let h = (seed ^ (seed >> 32)) as u32;
    h.wrapping_add(k.wrapping_mul(0x9E37_79B1)) ^ k.wrapping_mul(0x85EB_CA6B)
}

/// The noise fields that shape the land and the sea floor.
#[derive(Clone)]
pub struct Shape {
    warp: Perlin,
    coast: Perlin,
    land: Fbm<Perlin>,
    bridge: Perlin,
    stack: Perlin,
    cliff: Perlin,
    relief: Relief,
    seabed: Seabed,
}

impl Shape {
    /// Builds the noise fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self {
            warp: Perlin::new(sub_seed(seed, 1)),
            coast: Perlin::new(sub_seed(seed, 8)),
            land: Fbm::<Perlin>::new(sub_seed(seed, 2))
                .set_octaves(4)
                .set_frequency(1.0 / 640.0),
            bridge: Perlin::new(sub_seed(seed, 9)),
            stack: Perlin::new(sub_seed(seed, 10)),
            cliff: Perlin::new(sub_seed(seed, 11)),
            relief: Relief::new(seed),
            seabed: Seabed::new(seed),
        }
    }

    /// Returns the unwarped continent measure at `p`: above 0 on big land, below 0 at sea.
    pub fn base<P: Point>(&self, p: P) -> f32 {
        p.get(&self.land) as f32 - LAND_LEVEL
    }

    /// Returns `p` displaced by the large warp and the small coast warp, back on its surface.
    fn warped<P: Point>(&self, p: P) -> P {
        P::each(|k| {
            p.axis(k)
                + 140.0 * p.mul(0.0035).add(WARP_SHIFT[k]).get(&self.warp)
                + 26.0 * p.div(110.0).add(COAST_SHIFT[k]).get(&self.coast)
        })
        .onto(p)
    }

    /// Returns the sandbar and sea stack weights at the warped position, both only near big land.
    ///
    /// A field is read only where its weight is not zero, which changes no value.
    fn islets<P: Point>(&self, w: P, base: f32) -> [f32; 2] {
        let near = smooth(-0.4, -0.15, base);
        let ring = smooth(-0.42, -0.2, base) * smooth(0.1, 0.0, base);
        let line = |w: P| w.div(380.0).get(&self.bridge).abs() as f32;
        let pillar = |w: P| w.div(55.0).get(&self.stack) as f32;
        [
            if near > 0.0 {
                0.3 * near * smooth(0.1, 0.0, line(w))
            } else {
                0.0
            },
            if ring > 0.0 {
                ring * smooth(0.4, 0.7, pillar(w))
            } else {
                0.0
            },
        ]
    }

    /// Returns the ground height at `p`: sea floor, shelf, beach or cliff, then relief inland.
    ///
    /// Far out at sea the cliff weight cannot reach the floor, so its field is not read there.
    pub fn elevation<P: Point>(&self, p: P, climate: &Climate) -> f32 {
        let w = self.warped(p);
        let base = w.get(&self.land) as f32 - LAND_LEVEL;
        let [bar, stack] = self.islets(w, base);
        let s = base + bar + 0.3 * stack;
        let cliff = if s > CLIFF_REACH {
            smooth(0.1, 0.4, w.div(700.0).get(&self.cliff) as f32).max(stack)
        } else {
            0.0
        };
        if s > 0.0 {
            let c = Coast {
                s,
                base,
                cliff,
                stack,
            };
            return self.land_height(w, c);
        }
        let warmth = if s > -0.2 { climate.warmth(p) } else { 0.0 };
        self.seabed.floor(w, -s, cliff, warmth)
    }

    /// Returns the height on land at the warped position `w`, blending beach and cliff coasts.
    fn land_height<P: Point>(&self, w: P, c: Coast) -> f32 {
        let beach = 3.2 * smooth(0.0, 0.22, c.s);
        let bluff = (18.0 - 9.0 * c.stack) * smooth(0.0, 0.03, c.s);
        let coast = beach + (bluff - beach) * c.cliff;
        if c.base < 0.05 {
            return coast;
        }
        coast + self.relief.above(w, c.base)
    }
}

/// The coast measures at a point: `s` with islets and bars, `base` without, cliff and stack weights.
struct Coast {
    /// Inland measure including sandbars and sea stacks.
    s: f32,
    /// Inland measure of the continent alone.
    base: f32,
    /// Weight of the cliff coast profile.
    cliff: f32,
    /// Weight of the sea stack at this point.
    stack: f32,
}
