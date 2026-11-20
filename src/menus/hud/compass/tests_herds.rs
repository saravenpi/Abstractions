//! Tests of the herd markers: a herd the robot has seen gets a marker on the planet the robot stands
//! on, named "Herd" until its species is in the field guide, fading out with the time since it was
//! seen.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::herds::herds;
use crate::fauna::guide::Seen;
use crate::fauna::tune::MEMORY;
use crate::fauna::{Nearby, Sighting};
use crate::life::SpeciesId;
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Frame};
use crate::universe::Universe;

/// Returns the herd markers written for `nearby` seen by `seen`, with the robot on the home planet.
fn marks(nearby: Nearby, seen: Seen) -> Vec<crate::menus::Marker> {
    let body = Universe::new(1).home();
    let mut app = App::new();
    app.insert_resource(nearby)
        .insert_resource(seen)
        .init_resource::<Markers>()
        .insert_resource(Anchor {
            frame: Frame::Surface(body),
            origin: DVec3::ZERO,
            rotation: DQuat::IDENTITY,
            flat: false,
        });
    app.world_mut()
        .run_system_once(herds)
        .expect("the herd markers");
    let markers = app.world().resource::<Markers>().0.clone();
    markers
        .into_iter()
        .filter(|m| m.kind == MarkerKind::Herd)
        .collect()
}

/// A herd seen 200 m away on the robot's planet is marked as a herd, then by its name once in the
/// field guide; a herd on another planet is not marked.
#[test]
fn a_near_herd_is_marked() {
    let home = Universe::new(1).home();
    let species = SpeciesId {
        planet: 7,
        index: 3,
    };
    let sighting = Sighting {
        id: 9,
        at: DVec3::new(200.0, 6000.0, 0.0),
        species,
        name: "Kruseth".into(),
        metres: 200.0,
        seen: 0.0,
    };
    let nearby = |body| Nearby {
        body,
        herds: vec![sighting.clone()],
    };
    let plain = marks(nearby(Some(home)), Seen::default());
    assert_eq!(plain.len(), 1);
    assert_eq!(plain[0].label, "Herd");
    assert_eq!(plain[0].place, MarkerPlace::Body(home, sighting.at));
    let mut seen = Seen::default();
    seen.mark(species);
    assert_eq!(marks(nearby(Some(home)), seen)[0].label, "Kruseth");
    assert!(marks(nearby(None), Seen::default()).is_empty());
}

/// A herd seen half its memory ago is marked at half the strength of a fresh one.
#[test]
fn an_old_sighting_fades() {
    let home = Universe::new(1).home();
    let sighting = Sighting {
        id: 9,
        at: DVec3::new(200.0, 6000.0, 0.0),
        species: SpeciesId {
            planet: 7,
            index: 3,
        },
        name: "Kruseth".into(),
        metres: 200.0,
        seen: 0.0,
    };
    let fresh = super::herds::mark(&sighting, home, (false, 0.0))
        .color
        .alpha();
    let half = super::herds::mark(&sighting, home, (false, MEMORY * 0.5))
        .color
        .alpha();
    assert!((half - 0.5 * fresh).abs() < 1e-4, "{half} of {fresh}");
}
