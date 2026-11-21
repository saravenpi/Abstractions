//! Tests of the density fields: the plane and the globe read them alike, and no field shows a seam
//! between two faces of a globe.

use std::sync::Arc;

use bevy::prelude::*;

use super::density::Fields;
use crate::world::Terrain;
use crate::world::globe::{Face, Globe};

/// The flat view reads the plane, and two face charts give the same density and the same lode on
/// their shared edge.
#[test]
fn density_has_no_seam() {
    let fields = Fields::new(7);
    let w = Vec2::new(130.5, -77.25);
    let flat = Terrain::new(7);
    assert_eq!(
        fields.density((&flat, w), 2.0).grove,
        fields.density(w, 2.0).grove
    );
    assert_eq!(fields.lode((&flat, w)), fields.lode(w));
    let globe = Arc::new(Globe::new(7, 6000.0, 22.0));
    let view = |face| Terrain::on(globe.clone(), globe.face_chart(face));
    let (a, b) = (view(Face::PosY), view(Face::PosX));
    for i in 0..200 {
        let dir = Face::PosY.dir(0.999, -0.95 + 0.0095 * f64::from(i));
        let (pa, pb) = (a.chart().locate(dir), b.chart().locate(dir));
        let (da, db) = (fields.density((&a, pa), 8.0), fields.density((&b, pb), 8.0));
        let lode = (fields.lode((&a, pa)) - fields.lode((&b, pb))).abs();
        let gap = (da.grove - db.grove)
            .abs()
            .max((da.forest - db.forest).abs())
            .max(lode);
        assert!(
            gap < 1e-3,
            "density differs by {gap} across the edge at {dir}"
        );
    }
}
