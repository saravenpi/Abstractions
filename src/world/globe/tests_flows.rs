//! Tests of the flows: rivers fall all the way to the sea or a basin, are cut into the ground, and run only
//! on the worlds they belong to.

use super::Globe;
use super::atlas::{Cell, End, Flow, downhill};
use super::cube::Face;
use super::tests_landform::world;
use crate::universe::{PlanetClass, Water};
use crate::world::{SEA_LEVEL, Surface};

/// Returns every flow of `globe`, cell by cell over the six faces.
fn flows(globe: &Globe) -> Vec<Flow> {
    let n = globe.cells();
    let cells = Face::ALL
        .into_iter()
        .flat_map(move |face| (0..n).flat_map(move |i| (0..n).map(move |j| Cell { face, i, j })));
    cells
        .flat_map(|c| globe.sources(c).as_ref().clone())
        .collect()
}

/// Every river of two home planets falls at each step, its bed too, and ends at or under the sea or in a
/// hollow whose every neighbour is higher.
#[test]
fn rivers_fall_to_the_sea_or_a_basin() {
    for seed in [42, 7] {
        let globe = Globe::new(seed, 6000.0, 22.0);
        let rivers = flows(&globe);
        assert!(rivers.len() >= 3, "seed {seed} has {} rivers", rivers.len());
        for river in &rivers {
            let falls = river.nodes.windows(2);
            assert!(
                falls
                    .clone()
                    .all(|w| w[1].height < w[0].height && w[1].bed < w[0].bed)
            );
            let last = river.nodes[river.nodes.len() - 1];
            match river.end {
                End::Sea => assert!(last.height <= SEA_LEVEL, "{last:?}"),
                End::Basin => assert!(downhill(&globe, last.dir).1 > last.height - 0.05),
            }
        }
    }
}

/// The ground never stands above a river bed along the river, and above the sea it is a river bed, which
/// the meshes paint too.
#[test]
fn rivers_are_cut_into_the_ground() {
    let globe = Globe::new(42, 6000.0, 22.0);
    let mut beds = 0;
    for node in flows(&globe).iter().flat_map(|f| f.nodes.clone()) {
        let h = globe.height(node.dir);
        assert!(h <= node.bed.max(SEA_LEVEL) + 1e-3, "{h} over {node:?}");
        if h > SEA_LEVEL + 0.1 && !globe.pond_at(node.dir) {
            assert_eq!(globe.surface(node.dir), Surface::River, "{node:?}");
            assert_eq!(globe.mark(node.dir), Some(Surface::River));
            beds += 1;
        }
    }
    assert!(beds > 10, "only {beds} river beds above the sea");
}

/// Rivers run only on worlds of liquid water seas, molten rock runs in dark rock beds on a lava world,
/// and nothing flows on a desert, an ice world, a barren one or a dry world of islands.
#[test]
fn flows_follow_the_class() {
    let lava = world(PlanetClass::Lava, 0.25, 1400.0);
    let streams = flows(&lava);
    assert!(!streams.is_empty(), "no lava flows");
    for node in streams.iter().flat_map(|f| f.nodes.clone()) {
        if lava.height(node.dir) > SEA_LEVEL + 0.1 {
            assert_eq!(lava.surface(node.dir), Surface::Rock);
        }
    }
    let dry = [
        world(PlanetClass::Desert, 0.0, 320.0),
        world(PlanetClass::Ice, 0.0, 200.0),
        world(PlanetClass::Barren, 0.0, 250.0),
        world(PlanetClass::Island, 0.0, 288.0),
    ];
    for globe in &dry {
        assert!(
            flows(globe).is_empty(),
            "{:?} has flows",
            globe.record().class
        );
    }
    let frozen = Globe::new(42, 6000.0, 22.0).with_record(crate::universe::PlanetRecord {
        water: Water::Ice,
        ..crate::universe::PlanetRecord::TEMPERATE
    });
    assert!(
        flows(&frozen).is_empty(),
        "a sea that is not water has rivers"
    );
}
