//! Tests of the course marker: a course set to a star gives a marker the compass can aim at,
//! at any distance.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::aim::Around;
use super::beacons::course;
use crate::computer::{Destination, Target};
use crate::menus::hud::eye::{Eye, HudPose};
use crate::menus::{MarkerKind, Markers};
use crate::space::{Anchor, Clock, Frame};
use crate::universe::Universe;

/// Returns an app with the player in the space of the home star, a billion metres from it and
/// looking at it, with a course set to that star.
fn app() -> App {
    let universe = Universe::new(1);
    let star = universe.home().star;
    let anchor = Anchor {
        frame: Frame::System(star),
        origin: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        flat: false,
    };
    let eye = Eye {
        anchor,
        at: DVec3::new(1.0e9, 0.0, 0.0),
        forward: DVec3::NEG_X,
        up: DVec3::Y,
        body: None,
    };
    let mut app = App::new();
    app.insert_resource(universe)
        .insert_resource(Clock::default())
        .insert_resource(HudPose(Some(eye)))
        .insert_resource(Destination(Some(Target::Star(star))))
        .init_resource::<Markers>();
    app
}

/// A course to a star writes a course marker, and the compass reads it straight ahead at the
/// distance of the star.
#[test]
fn a_course_to_a_star_shows_on_the_compass() {
    let mut app = app();
    app.world_mut().run_system_once(course).expect("the course marker");
    let readings = app
        .world_mut()
        .run_system_once(|around: Around, markers: Res<Markers>| {
            markers
                .0
                .iter()
                .filter(|marker| marker.kind == MarkerKind::Course)
                .map(|marker| around.reading(marker))
                .collect::<Vec<_>>()
        })
        .expect("the compass reading");
    assert_eq!(readings.len(), 1);
    let (bearing, gap) = readings[0].expect("the course is on the compass");
    assert!(bearing.abs() < 1e-3, "{bearing}");
    assert!((gap - 1.0e9).abs() < 1.0e3, "{gap}");
}

/// Without a course no marker is written, and a cleared course removes the marker.
#[test]
fn no_course_no_marker() {
    let mut app = app();
    app.world_mut().run_system_once(course).expect("the course marker");
    app.world_mut().insert_resource(Destination(None));
    app.world_mut().run_system_once(course).expect("the course marker");
    let markers = app.world().resource::<Markers>();
    assert!(markers.0.iter().all(|marker| marker.kind != MarkerKind::Course));
}
