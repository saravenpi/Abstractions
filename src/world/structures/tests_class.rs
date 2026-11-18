//! Tests of who builds on each class of planet: only ruins on molten and bare rock, and nothing that needs
//! a sea on a dry world.

use std::sync::Arc;

use bevy::prelude::*;

use super::Terrain;
use super::site::{Kind, Site, structure_in_cell};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::globe::{Face, Globe};

/// Cells searched on each side of the middle of a face.
const SPAN: i32 = 9;

/// Returns the sites found around the middle of three faces of a 6 km globe of seed 11 with the home
/// record changed into `class` and an `ocean` share.
fn sites(class: PlanetClass, ocean: f64) -> Vec<Site> {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    let globe = Arc::new(Globe::new(11, 6000.0, 22.0).with_record(record));
    let cells = (-SPAN..SPAN).flat_map(|x| (-SPAN..SPAN).map(move |y| IVec2::new(x, y)));
    let faces = [Face::PosY, Face::PosX, Face::NegZ];
    let found = faces.into_iter().flat_map(|face| {
        let view = Terrain::on(globe.clone(), globe.face_chart(face));
        cells
            .clone()
            .filter_map(move |c| structure_in_cell(&view, c))
    });
    found.collect()
}

/// Returns how many of `sites` are of `kind`.
fn count(sites: &[Site], kind: Kind) -> usize {
    sites.iter().filter(|s| s.kind == kind).count()
}

/// Molten and bare rock have no village, no camp and no house: only ruins, and far fewer sites than home.
#[test]
fn lifeless_planets_keep_only_ruins() {
    let home = sites(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    assert!(count(&home, Kind::Village) > 0 && count(&home, Kind::Camp) > 0);
    for (class, ocean) in [(PlanetClass::Lava, 0.3), (PlanetClass::Barren, 0.0)] {
        let found = sites(class, ocean);
        let odd = found.iter().find(|s| !s.kind.ruin());
        assert!(odd.is_none(), "{class:?} has a {odd:?}");
        assert!(!found.is_empty(), "{class:?} lost its ruins");
        assert!(found.len() * 2 < home.len(), "{class:?}: {}", found.len());
    }
}

/// A dry world has no dock and no lighthouse, and the people of a desert or an ice world still build.
#[test]
fn a_dry_world_builds_nothing_by_the_sea() {
    let home = sites(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    assert!(count(&home, Kind::Dock) + count(&home, Kind::Lighthouse) > 0);
    for class in PlanetClass::ALL {
        let found = sites(class, 0.0);
        let by_sea = count(&found, Kind::Dock) + count(&found, Kind::Lighthouse);
        assert_eq!(by_sea, 0, "dry {class:?} builds by a sea it has not");
    }
    for class in [PlanetClass::Desert, PlanetClass::Ice] {
        let found = sites(class, 0.0);
        assert!(found.iter().any(|s| !s.kind.ruin()), "{class:?} is empty");
    }
}
