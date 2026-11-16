//! Land shape: warped continent mask with bays, islets, sandbars, cliffs and the elevation curves.

mod relief;
mod seabed;

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use super::biome::Climate;
use relief::Relief;
use seabed::Seabed;

/// Continent noise value above which the ground rises out of the sea.
const LAND_LEVEL: f32 = 0.16;

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

    /// Returns the unwarped continent measure at `(x, z)`: above 0 on big land, below 0 at sea.
    pub fn base(&self, x: f32, z: f32) -> f32 {
        self.land.get([x as f64, z as f64]) as f32 - LAND_LEVEL
    }

    /// Returns `(x, z)` displaced by the large warp and the small coast warp.
    fn warped(&self, x: f32, z: f32) -> [f64; 2] {
        let (px, pz) = (x as f64, z as f64);
        [
            px + 140.0 * self.warp.get([px * 0.0035, pz * 0.0035])
                + 26.0 * self.coast.get([px / 110.0, pz / 110.0]),
            pz + 140.0 * self.warp.get([px * 0.0035 + 31.7, pz * 0.0035 - 17.3])
                + 26.0 * self.coast.get([px / 110.0 + 9.1, pz / 110.0 - 4.3]),
        ]
    }

    /// Returns the sandbar and sea stack weights at the warped position, both only near big land.
    fn islets(&self, wx: f64, wz: f64, base: f32) -> [f32; 2] {
        let line = self.bridge.get([wx / 380.0, wz / 380.0]).abs() as f32;
        let bar = 0.3 * smooth(-0.4, -0.15, base) * smooth(0.1, 0.0, line);
        let ring = smooth(-0.42, -0.2, base) * smooth(0.1, 0.0, base);
        let pillar = self.stack.get([wx / 55.0, wz / 55.0]) as f32;
        [bar, ring * smooth(0.4, 0.7, pillar)]
    }

    /// Returns the ground height at `(x, z)`: sea floor, shelf, beach or cliff, then relief inland.
    pub fn elevation(&self, x: f32, z: f32, climate: &Climate) -> f32 {
        let [wx, wz] = self.warped(x, z);
        let base = self.land.get([wx, wz]) as f32 - LAND_LEVEL;
        let [bar, stack] = self.islets(wx, wz, base);
        let s = base + bar + 0.3 * stack;
        let cliff = smooth(0.1, 0.4, self.cliff.get([wx / 700.0, wz / 700.0]) as f32).max(stack);
        if s > 0.0 {
            let c = Coast {
                s,
                base,
                cliff,
                stack,
            };
            return self.land_height([wx, wz], c);
        }
        let warmth = if s > -0.2 { climate.warmth(x, z) } else { 0.0 };
        self.seabed.floor([wx, wz], -s, cliff, warmth)
    }

    /// Returns the height on land at the warped position `p`, blending beach and cliff coasts.
    fn land_height(&self, p: [f64; 2], c: Coast) -> f32 {
        let beach = 3.2 * smooth(0.0, 0.22, c.s);
        let bluff = (18.0 - 9.0 * c.stack) * smooth(0.0, 0.03, c.s);
        let coast = beach + (bluff - beach) * c.cliff;
        if c.base < 0.05 {
            return coast;
        }
        coast + self.relief.above(p[0], p[1], c.base)
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
