//! Plume width tests: whatever the thrust and the air, the bright core of a plume stays inside
//! its nozzle, the envelope stays within about two exit radii, and the envelope fades as it
//! widens, so the same gas gives the same light over a wider section.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::shape::shape;
use super::shells::jet_mesh;
use crate::ship::flight::BOOST;
use crate::ship::mesh::{EXIT, LIFT};

/// Thrusts tried: a whisper, a quarter, half, full throttle and the boost.
fn thrusts() -> [f32; 5] {
    [0.05, 0.25, 0.5, 1.0, (BOOST as f32).sqrt()]
}

/// Air tried, as a share of the air at sea level at home: from vacuum to sea level.
const AIRS: [f32; 5] = [0.0, 0.1, 0.3, 0.6, 1.0];

/// Returns the widest radius of the plume mesh, in shares of its unit radius at the nozzle: of
/// its bright core shell, and of the whole plume.
fn reach() -> (f32, f32) {
    let mesh = jet_mesh();
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("the plume mesh has no positions");
    };
    let Some(VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("the plume mesh has no colours");
    };
    let radius = |p: &[f32; 3]| Vec2::new(p[0], p[1]).length();
    let core = points
        .iter()
        .zip(colours)
        .filter(|(_, colour)| colour[0] == 0.0)
        .map(|(point, _)| radius(point))
        .fold(0.0, f32::max);
    (core, points.iter().map(radius).fold(0.0, f32::max))
}

/// At every thrust and in every air, the bright core of a main or lift plume is never wider
/// than its nozzle exit and the faint envelope never wider than 2.1 exit radii. The old plume
/// reached 3.2 exit radii in vacuum at full throttle and 4.2 with the boost.
#[test]
fn the_bright_core_stays_inside_the_nozzle() {
    let (core, all) = reach();
    for exit in [EXIT, LIFT.1] {
        for thrust in thrusts() {
            for air in AIRS {
                let plume = shape(exit, thrust, air);
                let (bright, faint) = (plume.width * core, plume.width * all);
                let at = format!("exit {exit} m, thrust {thrust}, air {air}");
                assert!(bright <= exit * 1.0001, "core {bright} m at {at}");
                assert!(faint <= 2.1 * exit, "envelope {faint} m at {at}");
            }
        }
    }
}

/// The envelope fades as one over the square of how much wider than in thick air it is, so the
/// light of a section stays the light of the same gas; and the boost makes a plume longer, not
/// wider.
#[test]
fn the_envelope_fades_as_it_widens() {
    for thrust in thrusts() {
        let thick = shape(EXIT, thrust, 1.0);
        for air in AIRS {
            let plume = shape(EXIT, thrust, air);
            assert!(plume.spread >= 1.0 && plume.spread <= 1.8, "{plume:?}");
            let base = plume.width / plume.spread;
            assert!((base - thick.width).abs() < 1.0e-6, "{plume:?} {thick:?}");
        }
    }
    for air in AIRS {
        let (full, boost) = (shape(EXIT, 1.0, air), shape(EXIT, thrusts()[4], air));
        assert_eq!(boost.width, full.width);
        assert!(boost.length > full.length);
    }
}
