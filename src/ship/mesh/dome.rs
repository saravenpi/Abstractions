//! The canopy and the spine behind it: one faceted shape that starts at the foot of the
//! windscreen, rises over the head of the pilot and runs back, narrowing, into the root of the
//! fin. Its front, over the cockpit, is the glass, which lifts on the hoop at its back. The rest
//! is the spine, a closed fairing of hull plates with a dark collar where the glass shuts on it.

use bevy::prelude::*;

use super::frame::{CANOPY, CANOPY_ENDS, GLAZED, SUNK};
use super::loft::{Section, cap, skin};
use super::shell::{lip, station};
use crate::ship::solid::Solid;

/// Returns the four corners of the section at a station of [`CANOPY`]: the right foot, the right
/// and the left edge of the roof, and the left foot. The feet are under the skin of the hull.
pub fn arch([z, foot, roof, top]: [f32; 4]) -> [Vec3; 4] {
    let low = lip(station(z), foot) - Vec3::Y * SUNK;
    [
        low,
        Vec3::new(roof, top, z),
        Vec3::new(-roof, top, z),
        Vec3::new(-low.x, low.y, z),
    ]
}

/// Returns the canopy glass: the windscreen, a fan from its foot, and the panes back to the
/// hoop. It is open underneath, over the cockpit, and at the back, where the spine closes it.
pub fn glass() -> Solid {
    let mut glass = Solid::default();
    let arches: Vec<[Vec3; 4]> = CANOPY[..GLAZED].iter().map(|&at| arch(at)).collect();
    let heart = Vec3::new(0.0, 0.4, 0.5 * (CANOPY[0][0] + CANOPY[GLAZED - 1][0]));
    for i in 0..3 {
        let screen = [CANOPY_ENDS.0, arches[0][i], arches[0][i + 1]];
        glass.tri(screen, (screen[1] + screen[2]) * 0.5 - heart);
        for pair in arches.windows(2) {
            let pane = [pair[0][i], pair[0][i + 1], pair[1][i + 1], pair[1][i]];
            glass.quad(pane, (pane[0] + pane[2]) * 0.5 - heart);
        }
    }
    glass
}

/// Returns the spine: the fairing from the hoop of the canopy back to its point in the root of
/// the fin. Its front wall is the bulkhead behind the head of the pilot, and its first span is
/// the dark collar.
pub fn spine() -> Solid {
    let sections: Vec<Section> = CANOPY[GLAZED - 1..]
        .iter()
        .map(|&at| arch(at).into())
        .collect();
    let mut spine = Solid::default();
    skin(&mut spine, &sections, false, |span, _| if span == 0 { 3 } else { 0 });
    spine.set_tone(3);
    cap(&mut spine, &sections[0], Vec3::NEG_Z);
    spine.set_tone(0);
    let last = &sections[sections.len() - 1];
    for i in 0..last.len() {
        let side = [CANOPY_ENDS.1, last[i], last[(i + 1) % last.len()]];
        spine.tri(side, (side[1] + side[2]) * 0.5 - super::loft::centre(last) + Vec3::Z);
    }
    spine
}

/// Returns the corner points of the canopy and of the spine together, for their collider.
pub fn corners() -> Vec<Vec3> {
    let arches = CANOPY.iter().flat_map(|&at| arch(at));
    arches.chain([CANOPY_ENDS.0, CANOPY_ENDS.1]).collect()
}

/// Returns the outline of the glass as a closed shape, with a floor between its feet and a wall
/// across the hoop: what the tests hold the lips of the mouth and the cabin against.
#[cfg(test)]
pub fn canopy() -> Solid {
    let sections: Vec<Section> = CANOPY[..GLAZED]
        .iter()
        .map(|&at| arch(at).into())
        .collect();
    let mut closed = Solid::default();
    skin(&mut closed, &sections, false, |_, _| 0);
    cap(&mut closed, &sections[GLAZED - 1], Vec3::Z);
    for i in 0..4 {
        let side = [CANOPY_ENDS.0, sections[0][i], sections[0][(i + 1) % 4]];
        closed.tri(side, Vec3::NEG_Z);
    }
    closed
}
