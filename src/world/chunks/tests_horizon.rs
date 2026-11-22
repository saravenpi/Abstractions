//! Tests of the horizon of the coarse terrain: what lies beneath the curve of the planet is
//! hidden, what stands over it is not, and the far nodes are measured right.

use std::sync::Arc;

use bevy::math::{DQuat, DVec3};

use super::build::coarse;
use super::horizon::{Bounds, CAST_REACH, Eye};
use crate::world::globe::quadtree::leaf_at;
use crate::world::globe::{Face, Globe};
use crate::world::{Terrain, globe::ChunkId};

/// Radius in metres of the planet of these tests: the Earth.
const RADIUS: f64 = 6.371e6;

/// Returns the bounds of a node `angle` radians wide whose middle lies `away` metres along the
/// ground from the north pole, with its highest point `high` metres over the sea.
fn node(away: f64, angle: f64, high: f64) -> Bounds {
    let turn = DQuat::from_rotation_x(away / RADIUS);
    Bounds {
        dir: turn * DVec3::Y,
        angle,
        radii: (RADIUS - 50.0, RADIUS + high),
    }
}

/// An eye `up` metres over the sea at the north pole, over a sea at sea level.
fn eye(up: f64) -> Eye {
    Eye {
        dir: DVec3::Y,
        height: RADIUS + up,
        floor: RADIUS,
    }
}

/// From 2 m up, the sea horizon is 5 km away: a flat node 20 km off is hidden, a 1,000 m ridge
/// as far stands over the curve, and a node at the feet is never hidden.
#[test]
fn the_curve_hides_low_far_nodes_and_not_high_ones() {
    let small = 200.0 / RADIUS;
    assert!(node(20_000.0, small, 0.0).hidden(&eye(2.0), false));
    assert!(!node(20_000.0, small, 1000.0).hidden(&eye(2.0), false));
    assert!(!node(3_000.0, small, 0.0).hidden(&eye(2.0), false));
    assert!(!node(0.0, small, 0.0).hidden(&eye(2.0), false));
    assert!(!node(20_000.0, small, 0.0).hidden(&eye(100.0), false));
}

/// A node wider than its gap to the eye is never hidden, nor anything for an eye under the floor.
#[test]
fn a_wide_node_or_a_low_eye_hides_nothing() {
    let wide = 30_000.0 / RADIUS;
    assert!(!node(20_000.0, wide, 0.0).hidden(&eye(2.0), false));
    let under = Eye {
        floor: RADIUS + 10.0,
        ..eye(2.0)
    };
    assert!(!node(20_000.0, 200.0 / RADIUS, 0.0).hidden(&under, false));
    assert!(!Bounds::WHOLE.hidden(&eye(2.0), false));
}

/// The nearest point of a node is its gap along the ground, and nothing for a node under the eye.
#[test]
fn the_nearest_point_is_the_gap() {
    let at = DVec3::Y * (RADIUS + 2.0);
    let near = node(0.0, 1000.0 / RADIUS, 0.0).nearest(at);
    assert!(near < 60.0, "{near}");
    let far = node(10_000.0, 1000.0 / RADIUS, 0.0).nearest(at);
    assert!((far - 9_000.0).abs() < 60.0, "{far}");
    assert!(far > CAST_REACH && near < CAST_REACH);
    assert!(Bounds::WHOLE.nearest(at) < 1.0e-6);
}

/// The bounds of a built node hold its ground and its corners.
#[test]
fn a_built_node_holds_its_ground() {
    let globe = Arc::new(Globe::new(7, 6000.0, 9.81));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let leaf = leaf_at(DVec3::Y, 3);
    let id = ChunkId {
        face: Face::PosY,
        ..leaf
    };
    let bounds = coarse(&terrain, id).bounds;
    let (low, high) = bounds.radii;
    assert!(low < globe.radius() + 1.0 && low < high, "{low} {high}");
    assert!(high >= globe.radius() - 2.0 && high < globe.radius() + 10_000.0);
    assert!(bounds.dir.angle_between(DVec3::Y) < 0.3);
    let width = std::f64::consts::FRAC_PI_2 / 8.0;
    assert!(
        bounds.angle > width * 0.5 && bounds.angle < width,
        "{}",
        bounds.angle
    );
}

/// A node at the threshold does not flash: once shown it is hidden only a little past the
/// horizon, and once hidden it stays hidden until it reaches the horizon again.
#[test]
fn a_node_at_the_horizon_does_not_flash() {
    let small = 200.0 / RADIUS;
    let eye = eye(2.0);
    let limit = (RADIUS / eye.height).acos() * RADIUS + 200.0;
    let just_past = node(limit * 1.02, small, 0.0);
    assert!(!just_past.hidden(&eye, false));
    assert!(just_past.hidden(&eye, true));
    let well_past = node(limit * 1.2, small, 0.0);
    assert!(well_past.hidden(&eye, false) && well_past.hidden(&eye, true));
}

/// A fading-out coarse node is never hidden abruptly by the horizon pass before its fade finishes.
#[test]
fn fading_node_is_not_hidden_by_horizon() {
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::*;

    use super::dither::Way;
    use super::horizon::hide;
    use super::nodes::Node;

    let mut world = World::new();
    let entity = world.spawn(Visibility::Inherited).id();
    let mut chunk = Node {
        entity,
        visible: true,
        bounds: Bounds::WHOLE,
        beyond: false,
        shadowless: false,
    };
    for way in [Some(Way::Hide), Some(Way::Leave)] {
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        hide(&mut commands, &mut chunk, true, way);
        queue.apply(&mut world);
        assert!(!chunk.beyond);
        assert_eq!(
            *world.get::<Visibility>(entity).unwrap(),
            Visibility::Inherited
        );
    }
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    hide(&mut commands, &mut chunk, true, None);
    queue.apply(&mut world);
    assert!(chunk.beyond);
    assert_eq!(
        *world.get::<Visibility>(entity).unwrap(),
        Visibility::Hidden
    );
}
