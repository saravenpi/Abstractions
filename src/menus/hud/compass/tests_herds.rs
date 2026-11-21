//! Tests of the herd markers: a herd drawn near the robot gets a marker on the planet the robot stands
//! on, at the middle of its animals and at full strength, named "Herd" until its species is in the
//! field guide.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::herds::herds;
use crate::fauna::guide::Seen;
use crate::fauna::{Nearby, Sighting};
use crate::life::SpeciesId;
use crate::menus::hud::context::{Context, Where};
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::space::{Anchor, Frame};
use crate::universe::Universe;

/// Returns the herd markers written for `nearby` seen by `seen`, with the robot on the home planet.
fn marks(nearby: Nearby, seen: Seen) -> Vec<crate::menus::Marker> {
    let body = Universe::new(1).home();
    let mut app = App::new();
    app.insert_resource(nearby)
        .insert_resource(seen)
        .insert_resource(Where(Context::OnFoot))
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

/// A herd drawn 200 m away on the robot's planet is marked as a herd, then by its name once in the
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

/// A herd's marker stands where its drawn animals are, at full strength whenever it is marked: there
/// is no fading memory of a herd that is not drawn.
#[test]
fn a_marker_stands_on_the_animals() {
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
    };
    let marker = super::herds::mark(&sighting, home, false);
    assert_eq!(marker.place, MarkerPlace::Body(home, sighting.at));
    assert_eq!(marker.color, super::herds::HERD);
}
