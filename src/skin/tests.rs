//! Tests of the shared shapes: each is closed, faces outward and keeps its size at both levels.

use bevy::prelude::*;

use super::rig::{floating, inside, open_edges, pieces, volume};
use super::{Detail, Disc, LARGE, Plate, Ring, Section, Soup, discs, loft, orb, plates};

/// A grey for shapes whose colour does not matter.
const GREY: [f32; 3] = [0.5; 3];

/// Returns one of each shape, apart from each other, at the level `detail`.
fn samples(detail: Detail) -> Soup {
    let mut soup = Soup::default();
    let plate = Plate::new([0.0, 0.0, 0.0], [0.1, 0.05, 0.02], [GREY; 2]).turned(0.3, 0.2, 0.1);
    let block = Plate::new([0.0, 0.5, 0.0], [0.1, 0.05, 0.04], [GREY; 2]).cut(LARGE);
    plates(&mut soup, &[plate, block], detail);
    let disc = Disc::new([1.0, 0.0, 0.0], Vec3::X, [0.05, 0.02], GREY);
    discs(&mut soup, &[disc], detail);
    orb(
        &mut soup,
        Vec3::X * 2.0,
        0.05,
        (detail.ring(Ring::Round(8, 2.0)), GREY),
    );
    let stack = [
        Section::new(3.0, 0.05, 0.04),
        Section::new(3.1, 0.07, 0.06).back(0.01),
        Section::new(3.2, 0.03, 0.03),
    ];
    for ring in [Ring::Round(12, 3.0), Ring::Cut(LARGE)] {
        loft(&mut soup, &stack, detail.ring(ring), |_, _| GREY);
    }
    soup
}

/// Every shape is closed and faces outward at both levels, and no triangle is degenerate.
#[test]
fn every_shape_is_closed_and_faces_outward() {
    for detail in [Detail::Near, Detail::Far] {
        let soup = samples(detail);
        for tri in soup.tris() {
            let area = (tri[1] - tri[0]).cross(tri[2] - tri[0]).length();
            assert!(area > 1e-8, "{detail:?}: {tri:?}");
        }
        for piece in pieces(&soup) {
            assert_eq!(open_edges(&piece), 0, "{detail:?}: {:?}", piece[0]);
            assert!(volume(&piece) > 1e-6, "{detail:?}: {:?}", piece[0]);
        }
    }
}

/// The far level has fewer triangles than the near one and fills the same box.
#[test]
fn the_far_level_is_lighter_and_as_large() {
    let (near, far) = (samples(Detail::Near), samples(Detail::Far));
    assert!(far.triangles() * 2 < near.triangles());
    let reach = |soup: &Soup| soup.points().fold(Vec3::ZERO, |high, p| high.max(p.abs()));
    assert!(reach(&near).distance(reach(&far)) < 0.02);
}

/// A chamfered box holds its middle and not a point beside it, and a mirrored soup is the same
/// shape on the other side, still facing outward.
#[test]
fn a_mirrored_soup_still_faces_outward() {
    let mut soup = Soup::default();
    let plate = Plate::new([0.3, 0.0, 0.0], [0.1, 0.05, 0.02], [GREY; 2]);
    plates(&mut soup, &[plate], Detail::Near);
    let mirrored = soup.clone().mirrored();
    let (here, there) = (pieces(&soup), pieces(&mirrored));
    assert!(inside(&here[0], Vec3::X * 0.3) && !inside(&here[0], Vec3::X * -0.3));
    assert!(inside(&there[0], Vec3::X * -0.3) && !inside(&there[0], Vec3::X * 0.3));
    assert!((volume(&here[0]) - volume(&there[0])).abs() < 1e-7 && volume(&there[0]) > 0.0);
}

/// Shapes that overlap are one piece, and a shape apart from the others floats.
#[test]
fn overlapping_shapes_are_one_piece() {
    let mut soup = Soup::default();
    let a = Plate::new([0.0, 0.0, 0.0], [0.1, 0.05, 0.02], [GREY; 2]);
    let b = Plate::new([0.15, 0.0, 0.0], [0.1, 0.02, 0.01], [GREY; 2]);
    plates(&mut soup, &[a, b], Detail::Near);
    assert!(floating(&pieces(&soup)).is_empty());
    plates(
        &mut soup,
        &[Plate::new([0.0, 1.0, 0.0], [0.1; 3], [GREY; 2])],
        Detail::Near,
    );
    assert!(!floating(&pieces(&soup)).is_empty());
}
