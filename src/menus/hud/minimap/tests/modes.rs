//! The modes of the minimap inside the app: its key cycles the ranges and switches it off,
//! and space swaps the ground for the chart of the system.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::read::{at, caption, drawn, part};
use super::rig::{app, radius, settle, tap};
use crate::computer::{Destination, Target};
use crate::menus::hud::minimap::scope::Scope;
use crate::menus::hud::minimap::space::{DASHES, Dash};
use crate::menus::hud::minimap::widget::Part;
use crate::space::{Anchor, Frame};
use crate::universe::{BodyId, Universe};

/// The default key of the minimap.
const KEY: KeyCode = KeyCode::KeyN;

/// Presses the minimap key, waits for the map to come to rest, and returns its range and
/// what the corner says.
fn cycled(app: &mut App) -> (f32, String) {
    tap(app, KEY);
    (settle(app), caption(app))
}

/// The key goes from the speed to each fixed range, the last one capped by the small planet.
/// The shown range eases to each step and a patch is painted for it.
#[test]
fn the_key_cycles_the_ranges() {
    let mut app = app("key");
    settle(&mut app);
    assert_eq!(cycled(&mut app), (250.0, "250 m".to_string()));
    tap(&mut app, KEY);
    let easing = app.world().resource::<Scope>().radius;
    assert!(easing > 250.0 && easing < 400.0, "{easing}");
    let wide = (settle(&mut app), caption(&mut app));
    assert_eq!(wide, (800.0, "800 m".to_string()));
    assert_eq!(part(&mut app, Part::Ground).width, percent(200));
    let capped = (radius() * 0.35) as f32;
    assert_eq!(cycled(&mut app), (capped, "2.10 km".to_string()));
}

/// After the last range the key switches the minimap off, and once more brings it back on
/// the range the speed asks for.
#[test]
fn the_key_switches_the_minimap_off_and_on() {
    let mut app = app("off");
    settle(&mut app);
    for _ in 0..4 {
        tap(&mut app, KEY);
    }
    assert_eq!(part(&mut app, Part::Frame).display, Display::None);
    assert_eq!(cycled(&mut app), (250.0, "250 m".to_string()));
    assert_eq!(part(&mut app, Part::Frame).display, Display::Flex);
    assert_eq!(part(&mut app, Part::Ground).display, Display::Flex);
}

/// Moves the world to the frame of the home system, 50 km from the home planet, or to the
/// frame of the galaxy, and runs half a second. Returns the home planet.
fn leave(app: &mut App, galaxy: bool) -> BodyId {
    let universe = app.world().resource::<Universe>();
    let home = universe.home();
    let origin = universe.planet_position(home, 0.0) + DVec3::X * 5.0e4;
    let frame = match galaxy {
        true => Frame::Galaxy(home.star.galaxy),
        false => Frame::System(home.star),
    };
    app.insert_resource(Anchor {
        frame,
        origin,
        rotation: DQuat::IDENTITY,
        flat: false,
    });
    for _ in 0..30 {
        app.update();
    }
    home
}

/// In space the ground and its marks give way to the chart of the system, named in the
/// corner, in a frame that did not move.
#[test]
fn space_shows_the_system_chart_in_the_same_frame() {
    let mut app = app("space");
    settle(&mut app);
    let (frame, empty) = (part(&mut app, Part::Frame), drawn(&mut app));
    let home = leave(&mut app, false);
    let system = app.world().resource::<Universe>().system(home.star);
    assert_eq!((empty, part(&mut app, Part::Frame)), (0, frame));
    assert_eq!(part(&mut app, Part::Ground).display, Display::None);
    let marks = [Part::Arrow, Part::Home, Part::Ship, Part::North];
    assert!(marks.into_iter().all(|mark| at(&mut app, mark).is_none()));
    let (charted, planets) = (drawn(&mut app), system.planets.len());
    assert!(
        charted >= 2 * planets + 2,
        "{charted} nodes, {planets} planets"
    );
    assert_eq!(caption(&mut app), system.star.name);
}

/// A destination in the system adds its course to the chart. Between the stars the chart of
/// the system the ship left stays, under the name of its star.
#[test]
fn the_chart_follows_the_destination_and_the_frame() {
    let mut app = app("course");
    settle(&mut app);
    let home = leave(&mut app, false);
    let charted = drawn(&mut app);
    let planets = app.world().resource::<Universe>().system(home.star).planets;
    let index = (0..planets.len() as u8).find(|index| *index != home.index);
    let target = Target::Body(BodyId {
        index: index.expect("a second planet"),
        ..home
    });
    app.insert_resource(Destination(Some(target)));
    leave(&mut app, false);
    assert_ne!(drawn(&mut app), charted);
    let mut dashes = app.world_mut().query_filtered::<(), With<Dash>>();
    assert_eq!(dashes.iter(app.world()).count(), DASHES - 1);
    leave(&mut app, true);
    let star = app.world().resource::<Universe>().system(home.star).star;
    assert!(drawn(&mut app) > 0, "the chart is gone between the stars");
    assert_eq!(caption(&mut app), star.name);
}
