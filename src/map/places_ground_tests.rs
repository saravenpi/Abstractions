//! Tests of where a pin lies: on the ground under the place it was dropped at, by the pin key at
//! the feet and by a right click on the flat map, on a planet of Earth's radius, far from the
//! middle of a cube face and near the seam between two faces; its marker reads the true distance,
//! and the save keeps it.

use std::f64::consts::TAU;
use std::sync::Arc;

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::{floor, ground, settle};
use crate::computer::SIZE;
use crate::map::geo::{from_chart, to_chart, wrapped};
use crate::map::level::{CLOSEST, REFERENCE_MAP};
use crate::map::link::{marks_of, marks_text};
use crate::map::places::keys::press;
use crate::map::places::{Places, fill};
use crate::map::sight::Sight;
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::saves::WorldSave;
use crate::universe::{BodyId, EARTH_RADIUS, STANDARD_GRAVITY, Universe};
use crate::world::{Globe, Planet};

/// Height in metres of the body of the player over the ground it stands on.
const STANDING: f64 = 1.0;

/// Directions far from the middle of every face: on the seam of the +X and +Y faces, by the
/// corner of three faces, and at a middle latitude of the south.
fn spots() -> [DVec3; 3] {
    [
        DVec3::new(1.0, 1.0, 0.37).normalize(),
        DVec3::new(1.0, 0.999, -1.0).normalize(),
        DVec3::new(-0.3, -0.62, 0.71).normalize(),
    ]
}

/// Returns a headless app that streams the ground of `home`, `globe`, in `universe`, and gives
/// the pins their height and writes their markers as the game does.
fn streaming(universe: Universe, home: BodyId, globe: Arc<Globe>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Markers>()
        .init_resource::<Places>()
        .insert_resource(universe)
        .insert_resource(Planet {
            body: home,
            globe,
            day_seconds: 86_400.0,
        })
        .add_systems(Update, (settle, fill).chain());
    app
}

/// The pin key at the feet of the player puts the pin on the ground under them, not at sea
/// level under the ground: within half a metre of the ground and two metres of the body.
#[test]
fn the_pin_key_drops_on_the_ground_at_the_feet() {
    let (globe, body) = (
        Globe::new(7, EARTH_RADIUS, STANDARD_GRAVITY),
        BodyId::default(),
    );
    for dir in spots() {
        let soil = dir * (EARTH_RADIUS + floor(&globe, dir));
        let feet = soil + dir * STANDING;
        let mut places = Places::default();
        press(&mut places, (body, feet), EARTH_RADIUS);
        assert!(ground(&mut places, body, &globe));
        let at = places.pins[0].point(EARTH_RADIUS);
        assert!(
            (at - soil).length() < 0.5,
            "{dir}: {}",
            (at - soil).length()
        );
        assert!(
            (at - feet).length() < 2.0,
            "{dir}: {}",
            (at - feet).length()
        );
        assert!(!ground(&mut places, body, &globe));
    }
}

/// Returns the direction a right click picks on the flat map looked at through `zoom`, with the
/// pointer on the window pixel nearest to where `dir` is drawn, off the middle of the map.
fn clicked(dir: DVec3, zoom: f32) -> DVec3 {
    let lens = Sight {
        zoom,
        centre: to_chart(dir) + DVec2::new(0.31, -0.27) / f64::from(zoom),
    };
    let scale = REFERENCE_MAP / SIZE;
    let pixel = (lens.to_map(to_chart(dir)) * scale).round() / scale;
    from_chart(wrapped(lens.to_chart(pixel))).expect("on the map")
}

/// A right click on the map at the closest zoom and at a middle zoom drops the pin on the ground
/// under the pointer: the place it marks is within the ground one window pixel of the map covers,
/// and it lies on the ground there.
#[test]
fn a_click_on_the_map_drops_on_the_ground_under_the_pointer() {
    let (globe, body) = (
        Globe::new(7, EARTH_RADIUS, STANDARD_GRAVITY),
        BodyId::default(),
    );
    for zoom in [CLOSEST, 512.0] {
        let pixel = TAU * EARTH_RADIUS / f64::from(zoom * REFERENCE_MAP);
        for dir in spots() {
            let picked = clicked(dir, zoom);
            let soil = picked * (EARTH_RADIUS + floor(&globe, picked));
            let mut places = Places::default();
            places.drop(body, picked);
            ground(&mut places, body, &globe);
            let off = picked.angle_between(dir) * EARTH_RADIUS;
            assert!(off < pixel, "{dir} at {zoom}: {off} m, a pixel {pixel} m");
            assert!((places.pins[0].point(EARTH_RADIUS) - soil).length() < 0.5);
        }
    }
}

/// A pin dropped on the map of another planet keeps no height until the world is on its planet.
#[test]
fn a_pin_of_another_body_waits_for_its_ground() {
    let (globe, here) = (
        Globe::new(7, EARTH_RADIUS, STANDARD_GRAVITY),
        BodyId::default(),
    );
    let mut places = Places::default();
    places.drop(BodyId { moon: 1, ..here }, DVec3::X);
    assert!(!ground(&mut places, here, &globe));
    assert_eq!(places.pins[0].height, None);
}

/// The marker of a pin just dropped at the feet of the player, on the home planet the world
/// streams, is under two metres from the player.
#[test]
fn the_marker_of_a_pin_at_the_feet_reads_under_two_metres() {
    let universe = Universe::new(3);
    let home = universe.home();
    let radius = universe.planet(home).radius;
    let globe = Arc::new(Globe::new(3, radius, STANDARD_GRAVITY));
    let dir = spots()[0];
    let feet = dir * (radius + floor(&globe, dir) + STANDING);
    let mut app = streaming(universe, home, globe);
    press(
        &mut app.world_mut().resource_mut::<Places>(),
        (home, feet),
        radius,
    );
    app.update();
    let markers = &app.world().resource::<Markers>().0;
    let pin = markers
        .iter()
        .find(|marker| marker.kind == MarkerKind::Favourite);
    let Some(MarkerPlace::Body(body, at)) = pin.map(|marker| marker.place) else {
        panic!("no pin marker in {markers:?}");
    };
    assert_eq!(body, home);
    assert!((at - feet).length() < 2.0, "{}", (at - feet).length());
}

/// The world file and the record of the server keep the pin where it lies, and a pin of an
/// older file, which has no height, loads with none.
#[test]
fn the_save_keeps_the_pin_on_the_ground() {
    let (globe, body) = (
        Globe::new(7, EARTH_RADIUS, STANDARD_GRAVITY),
        BodyId::default(),
    );
    let mut places = Places::default();
    places.drop(body, spots()[1]);
    ground(&mut places, body, &globe);
    let save = WorldSave {
        marks: places.pins.clone(),
        ..WorldSave::default()
    };
    let back: WorldSave = toml::from_str(&toml::to_string(&save).unwrap()).unwrap();
    let kept = places.pins[0].point(EARTH_RADIUS);
    assert!((back.marks[0].point(EARTH_RADIUS) - kept).length() < 0.5);
    let record = marks_of(&marks_text(&places.pins));
    assert!((record[0].point(EARTH_RADIUS) - kept).length() < 0.5);
    let old = marks_of("[[mark]]\nname = \"Camp\"\ndir = [1.0, 0.0, 0.0]\n");
    assert_eq!(old[0].height, None);
}
