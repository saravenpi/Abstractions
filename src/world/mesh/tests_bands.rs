//! Tests of the deck of a gas giant: a smooth sphere on its radius, in soft bands of its two tones.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::bands::band;
use super::node_mesh;
use crate::universe::PlanetClass;
use crate::world::Terrain;
use crate::world::globe::far::bake;
use crate::world::globe::tests_deck::giant;
use crate::world::globe::{ChunkId, Face};

/// Returns the positions of `mesh`.
fn positions(mesh: &Mesh) -> Vec<Vec3> {
    let found = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
    let all = found
        .and_then(|values| values.as_float3())
        .expect("positions");
    all.iter().map(|p| Vec3::from_array(*p)).collect()
}

/// The far sphere of a giant has every vertex on its deck, and a deck node every vertex but those of
/// its skirts.
#[test]
fn a_giant_is_a_smooth_sphere() {
    let deck = Arc::new(giant());
    let radius = deck.radius() as f32;
    let far = positions(&bake(&deck));
    assert!(far.iter().all(|p| (p.length() - radius).abs() < 0.1));
    let id = ChunkId {
        face: Face::NegX,
        level: deck.deepest_level(),
        x: 7,
        y: 30,
    };
    let terrain = Terrain::on(deck.clone(), deck.face_chart(id.face));
    let node = positions(&node_mesh(&terrain, id));
    let top = 16 * 16 * 2 * 3;
    let centre = super::node_frame(&terrain, id).local(DVec3::ZERO);
    assert!(
        node[..top]
            .iter()
            .all(|p| ((*p - centre).length() - radius).abs() < 0.1)
    );
    assert!(node.len() > top);
}

/// The bands blend the two tones of the look, change with the latitude, and the same direction always
/// gets the same colour.
#[test]
fn a_giant_is_banded_in_its_tones() {
    let deck = giant();
    let look = PlanetClass::GasGiant.look();
    let (low, high) = (look.sea.min(look.land), look.sea.max(look.land));
    let around = |y: f64| DVec3::new((1.0 - y * y).sqrt(), y, 0.0);
    let colors: Vec<Vec3> = (0..=40)
        .map(|k| Vec3::from_slice(&band(&deck, around(k as f64 / 20.0 - 1.0))))
        .collect();
    for c in &colors {
        assert!(
            c.cmpge(low - 1e-5).all() && c.cmple(high + 1e-5).all(),
            "{c}"
        );
    }
    let spread = colors
        .iter()
        .map(|c| c.distance(look.sea))
        .fold(0.0, f32::max);
    assert!(spread > 0.5 * look.land.distance(look.sea), "{spread}");
    assert_eq!(band(&deck, around(0.3)), band(&deck, around(0.3)));
}
