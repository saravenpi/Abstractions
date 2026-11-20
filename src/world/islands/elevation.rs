//! The ground height: the warped coast, its islets, beaches and cliffs, the sea floor and the relief inland.

use super::{CLIFF_REACH, COAST_SHIFT, LAND_LEVEL, Point, Shape, WARP_SHIFT, smooth};
use crate::world::biome::Climate;

impl Shape {
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
        self.measure(p, climate).0
    }

    /// Returns the ground height at `p` and the continent measure at its warped position, which follows
    /// the coast.
    pub fn measure<P: Point>(&self, p: P, climate: &Climate) -> (f32, f32) {
        self.measure_on(p, climate, 0.0)
    }

    /// Returns the ground height at `p` and the continent measure at its warped position, with `bias`
    /// added to the measure: what a larger shape of the ground, a continent or an ocean of the plates, says
    /// about the place. A bias of 1 leaves only land and one of -1 only open sea; in between the coast
    /// keeps every shape it has, moved towards one side.
    pub fn measure_on<P: Point>(&self, p: P, climate: &Climate, bias: f32) -> (f32, f32) {
        let w = self.warped(p);
        let base = self.land(w) - LAND_LEVEL + bias;
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
            return (self.land_height(w, c), base);
        }
        let warmth = if s > -0.2 { climate.warmth(p) } else { 0.0 };
        (self.seabed.floor(w, -s, cliff, warmth), base)
    }

    /// Returns the height on land at the warped position `w`, blending beach and cliff coasts.
    fn land_height<P: Point>(&self, w: P, c: Coast) -> f32 {
        let beach = 3.2 * smooth(0.0, 0.22, c.s);
        let bluff = (18.0 - 9.0 * c.stack) * smooth(0.0, 0.03, c.s);
        let coast = beach + (bluff - beach) * c.cliff;
        if c.base < 0.05 {
            return coast;
        }
        let gain = match P::SOLID {
            true => self.gain,
            false => 1.0,
        };
        coast + self.relief.above(w, c.base, gain)
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
