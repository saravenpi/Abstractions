//! Tests of who builds where the climate allows: villages only in air they can live in, and no camp
//! in the heart of a desert.

use std::sync::Arc;

use bevy::prelude::*;

use super::Terrain;
use super::site::{Kind, Site, structure_in_cell};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::globe::{Face, Globe};
use crate::world::lands::thrive::{climate_at, dune_sea, livable};

/// Cells searched on each side of the middle of a face.
const SPAN: i32 = 9;

/// Returns the sites found around the middle of three faces of a 6 km globe of seed 11 with the home
/// record changed into `class`, `ocean` and `temperature` kelvin, each with the view it lies in.
fn sites(class: PlanetClass, ocean: f64, temperature: f64) -> Vec<(Terrain, Site)> {
    let record = PlanetRecord {
        class,
        ocean,
        temperature,
        ..PlanetRecord::TEMPERATE
    };
    let globe = Arc::new(Globe::new(11, 6000.0, 22.0).with_record(record));
    let cells = (-SPAN..SPAN).flat_map(|x| (-SPAN..SPAN).map(move |y| IVec2::new(x, y)));
    let faces = [Face::PosY, Face::PosX, Face::NegZ];
    let found = faces.into_iter().flat_map(|face| {
        let view = Terrain::on(globe.clone(), globe.face_chart(face));
        let each = cells.clone().filter_map(|c| structure_in_cell(&view, c));
        each.map(|site| (view.clone(), site)).collect::<Vec<_>>()
    });
    found.collect()
}

/// Every village stands in air between its coldest and warmest, on a world as cold, as mild and as
/// hot as several records give. The mild home world, searched first, still has villages, and the
/// memory of cells keeps the planets apart: none of them reuses a village of home.
#[test]
fn villages_keep_to_their_air() {
    let home = sites(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN, 288.0);
    assert!(home.iter().any(|(_, s)| s.kind == Kind::Village));
    for temperature in [250.0, 288.0, 330.0] {
        let found = sites(
            PlanetClass::Island,
            PlanetRecord::BALANCED_OCEAN,
            temperature,
        );
        for (view, site) in found.iter().filter(|(_, s)| s.kind == Kind::Village) {
            let air = climate_at(view, site.origin).expect("a globe has a climate");
            assert!(livable(&air), "a village in {} K", air.temperature);
        }
    }
}

/// No camp stands in the heart of a desert, on a mild and on a hot dry desert world, and the
/// mild one still has camps out of that heart.
#[test]
fn nobody_camps_in_the_heart_of_a_desert() {
    let mut kept = 0;
    for temperature in [288.0, 320.0] {
        let found = sites(PlanetClass::Desert, 0.0, temperature);
        let camps: Vec<_> = found.iter().filter(|(_, s)| s.kind == Kind::Camp).collect();
        for (view, site) in &camps {
            assert!(!dune_sea(view, site.origin), "a camp at {}", site.origin);
        }
        kept += usize::from(temperature < 300.0) * camps.len();
    }
    assert!(kept > 0, "the mild desert lost its camps");
}
