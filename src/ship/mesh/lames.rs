//! The layered plates of the hull: three lames over each wing root, each overlapping the next
//! like the plates of a shoulder guard, on a dark plate of the frame that shows round them, and
//! three bands round each engine nacelle. A triangle of the accent colour marks the middle lame
//! of each wing. Every plate is sunk into the part that carries it.

use bevy::prelude::*;

use super::loft::{Section, round, solid};
use super::tail::{ENGINES, ring_radius};
use super::wing::{STATIONS, top};
use crate::ship::solid::Solid;

/// Where along the span each lame of a wing root starts and ends, in metres from the middle, and
/// how far it stands over the wing. The first is the highest and overlaps the second.
const LAMES: [(f32, f32, f32); 3] = [(0.9, 1.34, 0.055), (1.26, 1.7, 0.04), (1.62, 2.06, 0.025)];

/// The shares of the chord the lames cover, from the leading edge.
const CHORD: (f32, f32) = (0.27, 0.62);

/// How far a plate is sunk into the wing, and how far the dark plate of the frame shows round
/// the lames, in metres.
const SEAT: (f32, f32) = (0.03, 0.05);

/// Where along z the bands of an engine nacelle start, their length and how far they stand out.
const BANDS: ([f32; 3], f32, f32) = ([2.3, 2.6, 2.9], 0.22, 0.03);

/// Returns the leading edge and the chord of the wing `x` metres out, between its stations.
fn chord_at(x: f32) -> (f32, f32) {
    let i = STATIONS.iter().rposition(|s| s[0] <= x).unwrap_or(0).min(2);
    let (a, b) = (STATIONS[i], STATIONS[i + 1]);
    let t = (x - a[0]) / (b[0] - a[0]);
    (a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t)
}

/// Returns the point of the top of the right wing `x` out and `share` of the chord back.
fn on_top(x: f32, share: f32) -> Vec3 {
    let (front, chord) = chord_at(x);
    Vec3::new(x, top(x), front + share * chord)
}

/// Adds a plate on the top of the right wing between the spans `x` and the chord shares `chord`,
/// sunk into the wing and standing `lift` metres over it.
fn tile(solid: &mut Solid, x: (f32, f32), chord: (f32, f32), lift: f32) {
    let base = [(x.0, chord.0), (x.1, chord.0), (x.1, chord.1), (x.0, chord.1)]
        .map(|(x, share)| on_top(x, share) - Vec3::Y * SEAT.0);
    solid.plate(base, Vec3::Y * (SEAT.0 + lift));
}

/// Adds a closed wedge: the triangle `base`, extruded by `depth`.
fn wedge(solid: &mut Solid, base: [Vec3; 3], depth: Vec3) {
    let top = base.map(|p| p + depth);
    let centre = (base[0] + base[1] + base[2]) / 3.0 + depth * 0.5;
    solid.tri(base, -depth);
    solid.tri(top, depth);
    for i in 0..3 {
        let j = (i + 1) % 3;
        let side = [base[i], base[j], top[j], top[i]];
        solid.quad(side, (base[i] + top[j]) * 0.5 - centre);
    }
}

/// Returns the plates of the root of the right wing: the three lames, the dark plate of the
/// frame under them, and the triangle of the accent colour on the middle lame, which points at
/// the nose.
fn root() -> [Solid; 3] {
    let [mut lames, mut under, mut mark] = [Solid::default(), Solid::default(), Solid::default()];
    for (k, (from, to, lift)) in LAMES.into_iter().enumerate() {
        lames.set_tone([1, 0, 1][k]);
        tile(&mut lames, (from, to), CHORD, lift);
    }
    let span = (LAMES[0].0, LAMES[2].1 + SEAT.1);
    let chord = (CHORD.0 - 0.02, CHORD.1 + 0.02);
    tile(&mut under, span, chord, 0.012);
    let (lift, x) = (LAMES[1].2, (LAMES[1].0 + 0.12, LAMES[1].1 - 0.08));
    let base = [(x.0, 0.5), (x.1, 0.5), ((x.0 + x.1) * 0.5, 0.36)]
        .map(|(x, share)| on_top(x, share) + Vec3::Y * (lift - 0.01));
    wedge(&mut mark, base, Vec3::Y * 0.02);
    [lames, under, mark]
}

/// Returns the bands round the engine nacelle on the axis `at`: rings wider than the nacelle,
/// chamfered at both ends into it, in two tones.
fn bands(at: Vec2) -> Solid {
    let mut bands = Solid::default();
    let (starts, length, out) = BANDS;
    for (k, z) in starts.into_iter().enumerate() {
        let stations = [
            (z, -0.05),
            (z + 0.03, out),
            (z + length - 0.03, out),
            (z + length, -0.05),
        ];
        let sections: Vec<Section> = stations
            .map(|(z, grow)| round(at.extend(z), ring_radius(z) + grow, 10))
            .into();
        solid(&mut bands, &sections, |_, _| [1, 0, 1][k]);
    }
    bands
}

/// Returns the plates of the airframe: the lames of the wing roots, the dark plates under them,
/// the triangles of the accent colour, and the bands of the engine nacelles.
pub fn lames() -> [Solid; 4] {
    let [lames, under, mark] = root();
    let mut rings = Solid::default();
    for at in ENGINES {
        rings.merge(&bands(at), Vec3::ZERO);
    }
    [lames.mirrored(), under.mirrored(), mark.mirrored(), rings]
}
