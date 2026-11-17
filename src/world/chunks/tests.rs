//! Tests of chunk streaming on the globe: the embedding, the draw cut, the focus face and frames.

use std::sync::Arc;

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::cut::visible;
use super::embed::Embedding;
use super::face::{next_face, rehome};
use super::near::Around;
use super::place::{Sited, pose};
use crate::space::Anchor;
use crate::universe::Universe;
use crate::world::focus::Link;
use crate::world::globe::quadtree::{Changes, Tree, children};
use crate::world::globe::{ChunkId, Face};
use crate::world::mesh::leaf_coord;
use crate::world::{Globe, Planet, Terrain};

/// Returns the view of a globe of `radius` metres through the chart of `face`.
pub fn view(radius: f64, face: Face) -> Terrain {
    let globe = Arc::new(Globe::new(9, radius, 22.0));
    Terrain::on(globe.clone(), globe.face_chart(face))
}

/// The draw cut shows the children of a split node only once all four have a mesh, and never
/// draws a node together with one that covers it.
#[test]
fn draw_cut_waits_for_all_four_children() {
    let root = ChunkId {
        face: Face::PosY,
        level: 0,
        x: 0,
        y: 0,
    };
    let mut tree = Tree::default();
    tree.apply(&Changes {
        splits: vec![root],
        merges: vec![],
    });
    let kids = children(root);
    let roots_only = visible(&tree, &|id| id.level == 0);
    assert_eq!(roots_only.len(), 6);
    let three = visible(&tree, &|id| id != kids[3]);
    assert!(three.contains(&root) && !three.contains(&kids[0]));
    let all = visible(&tree, &|_| true);
    assert!(!all.contains(&root) && kids.iter().all(|kid| all.contains(kid)));
    assert_eq!(all.len(), 9);
    assert!(visible(&tree, &|_| false).is_empty());
}

/// The terrain view changes face only once the focus is past the edge by the margin, and a
/// spawn found past the edge of a chart is moved to the face that holds it.
#[test]
fn terrain_view_follows_the_focus_face() {
    let terrain = view(6000.0, Face::PosY);
    let chart = terrain.chart();
    assert_eq!(next_face(chart, Face::PosY.dir(0.9, 0.2)), None);
    assert_eq!(next_face(chart, Face::PosY.dir(1.001, 0.2)), None);
    assert_eq!(
        next_face(chart, Face::PosY.dir(1.01, 0.2)),
        Some(Face::PosX)
    );
    assert_eq!(next_face(chart, DVec3::NEG_Y), Some(Face::NegY));
    let past = Vec3::new(4500.0, 3.0, 100.0);
    let (moved, at) = rehome(&terrain, past);
    assert_eq!(next_face(moved.chart(), moved.up(at.x, at.z)), None);
    assert!(
        moved
            .embed(at.x, at.y, at.z)
            .distance(terrain.embed(past.x, past.y, past.z))
            < 0.01
    );
    assert!(at.x.abs().max(at.z.abs()) <= 4096.0);
}

/// Around the focus, leaves of the focus face keep their own coordinates, and the leaves that
/// touch the focus leaf across a face edge are solid.
#[test]
fn leaves_across_a_face_edge_are_near() {
    let terrain = view(6000.0, Face::PosY);
    let level = terrain.globe().leaf_level();
    let last = (1u32 << level) - 1;
    let under = ChunkId {
        face: Face::PosY,
        level,
        x: last,
        y: last,
    };
    let around = Around::new(&terrain, under.dir(0.5, 0.5) * 6000.0).expect("a face chart");
    assert_eq!(around.center, leaf_coord(under));
    assert_eq!(around.coord(under), leaf_coord(under));
    assert_eq!(around.near.len(), 8);
    assert!(around.near.iter().all(|id| around.solid(*id)));
    let across = under
        .neighbor(1, 0)
        .expect("a side neighbour always exists");
    assert_ne!(across.face, Face::PosY);
    assert_eq!(
        (around.coord(across) - around.center).abs().max_element(),
        1
    );
}

/// On the surface frame a chunk renders where the anchor puts its planet position, and in the
/// frame of the star the planet frame turns with the day and comes back through the link.
#[test]
fn chunk_poses_follow_the_anchor_frame() {
    let terrain = view(6000.0, Face::PosY);
    let universe = Universe::new(3);
    let body = universe.home();
    let spawn = Vec3::new(40.0, 6.0, -25.0);
    let anchor = Anchor::surface(body, &terrain, spawn);
    let planet = Planet {
        body,
        globe: terrain.globe().clone(),
        day_seconds: 1200.0,
    };
    let on_foot = Link::between(anchor.frame, &planet, &universe, 77.0).expect("the surface frame");
    let sited = Sited::from(Embedding::of(&terrain, IVec2::ZERO).frame());
    let (placed, transform) = pose(&anchor, &on_foot, &sited);
    assert_eq!(placed.pos, sited.pos);
    assert!((crate::space::altitude(&anchor, terrain.globe(), transform.translation)).abs() < 1e-3);
    assert!(transform.translation.distance(spawn) < 100.0);
    assert!((transform.rotation * Vec3::Y).y > 0.9999);
    let frame = crate::space::Frame::System(body.star);
    let in_space = Link::between(frame, &planet, &universe, 300.0).expect("the frame of the star");
    let p = DVec3::new(6000.0, 10.0, 0.0);
    assert!(in_space.to_planet(in_space.to_anchor(p)).distance(p) < 1e-6);
    let east = (in_space.turn * DVec3::X).dot(DVec3::NEG_Z);
    assert!(
        (east - 1.0).abs() < 1e-9,
        "a quarter day turns +X to the east, -Z"
    );
    assert_eq!(in_space.turn * DQuat::IDENTITY, in_space.turn);
    let other = crate::space::Frame::Galaxy(body.star.galaxy);
    assert!(Link::between(other, &planet, &universe, 0.0).is_none());
}
