//! The layered plates of the hull: three lames on the top of each wing, three on each flank
//! of the fuselage and a band of them on each engine pod, each row stepping up a little from
//! the one in front like the plates of a guard. Under them the dark frame shows in the gaps,
//! and a thin line of glow runs along the last wing lame. Every plate is sunk into the part
//! that carries it.

use bevy::prelude::*;

use super::loft::{Section, round, solid};
use super::outside::layer;
use super::tail::{ENGINES, ring_radius};
use super::wing::{STATIONS, top};
use crate::ship::solid::Solid;

/// The shares of the chord each lame of a wing covers, from the leading edge: they overlap by a
/// few centimetres, and each stands 1.2 cm higher than the one in front.
const WING_LAMES: [(f32, f32); 3] = [(0.26, 0.44), (0.4, 0.58), (0.54, 0.72)];

/// Where along the span the lames of a wing start and end, in metres from the middle.
const SPAN: (f32, f32) = (0.7, 2.85);

/// Where the lames of the flank are along z, front and back, in metres.
const FLANK: [(f32, f32); 3] = [(-0.7, 0.05), (-0.02, 0.8), (0.75, 1.55)];

/// The stations of the crest: z and the height of its top.
const CREST: [(f32, f32); 4] = [(0.25, 0.8), (1.2, 0.95), (2.2, 0.98), (3.0, 0.8)];

/// Where along z the bands of an engine pod are, and their length.
const BANDS: ([f32; 3], f32) = ([2.55, 2.8, 3.05], 0.14);

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

/// Returns the three lames of the right wing and, in the second part, the line of glow along
/// the edge of the last one.
fn wing_lames() -> (Solid, Solid) {
    let (mut lames, mut line) = (Solid::default(), Solid::default());
    for (k, (from, to)) in WING_LAMES.into_iter().enumerate() {
        lames.set_tone([1, 2, 1][k]);
        let base = [(SPAN.0, from), (SPAN.1, from), (SPAN.1, to), (SPAN.0, to)]
            .map(|(x, share)| on_top(x, share) - Vec3::Y * 0.03);
        lames.plate(base, Vec3::Y * (0.03 + 0.012 * (k as f32 + 1.0)));
    }
    let (last, rise) = (WING_LAMES[2].1, 0.012 * 3.0);
    let base = [
        (1.0, last - 0.022),
        (2.7, last - 0.022),
        (2.7, last - 0.008),
        (1.0, last - 0.008),
    ]
    .map(|(x, share)| on_top(x, share) + Vec3::Y * (rise - 0.01));
    line.plate(base, Vec3::Y * 0.02);
    (lames, line)
}

/// Returns the lames of the left flank of the fuselage.
fn flank_lames() -> Solid {
    let mut lames = Solid::default();
    for (k, span) in FLANK.into_iter().enumerate() {
        lames.set_tone([2, 1, 2][k]);
        layer(&mut lames, span, (0.1, 0.78), 0.012 * (k as f32 + 1.0));
    }
    lames
}

/// Returns the bands round the engine pod on the axis `at`: rings a little wider than the pod,
/// chamfered at both ends into it, that step out one after the other toward the nozzle.
fn bands(at: Vec2) -> Solid {
    let mut bands = Solid::default();
    for (k, z) in BANDS.0.into_iter().enumerate() {
        let (low, wide) = (-0.05, 0.012 * (k as f32 + 1.0));
        let stations = [
            (z, low),
            (z + 0.03, wide),
            (z + BANDS.1 - 0.03, wide),
            (z + BANDS.1, low),
        ];
        let sections: Vec<Section> = stations
            .map(|(z, grow)| round(at.extend(z), ring_radius(z) + grow, 10))
            .into();
        solid(&mut bands, &sections, |_, _| 3);
    }
    bands
}

/// Returns the crest: a thin ridge along the spine from behind the canopy to the fin, rising
/// to it, chamfered along its top.
fn crest() -> Solid {
    let section = |(z, height): (f32, f32)| -> Section {
        [
            (-0.07, 0.5),
            (0.07, 0.5),
            (0.07, height - 0.04),
            (0.03, height),
            (-0.03, height),
            (-0.07, height - 0.04),
        ]
        .map(|(x, y)| Vec3::new(x, y, z))
        .into()
    };
    let sections: Vec<Section> = CREST.into_iter().map(section).collect();
    let mut crest = Solid::default();
    solid(&mut crest, &sections, |_, _| 3);
    crest
}

/// Returns `solid` and its mirror image on the other side of the ship.
fn both(solid: &Solid) -> Solid {
    let mut all = solid.clone();
    for (tri, tone) in solid.tris.iter().zip(&solid.tones) {
        let [a, b, c] = tri.map(|p| Vec3::new(-p.x, p.y, p.z));
        all.tris.push([a, c, b]);
        all.tones.push(*tone);
    }
    all
}

/// Returns the plates of the airframe: the lames of the wings and of the flanks, the line of
/// glow along the wings, the bands of the engine pods and the crest.
pub fn lames() -> [Solid; 4] {
    let (wing, line) = wing_lames();
    let mut plates = both(&wing);
    plates.merge(&both(&flank_lames()), Vec3::ZERO);
    let mut pods = Solid::default();
    for at in ENGINES {
        pods.merge(&bands(at), Vec3::ZERO);
    }
    [plates, both(&line), pods, crest()]
}
