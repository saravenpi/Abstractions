//! The far limit of the markers over the world, in the whole HUD: on foot a pin nobody follows
//! 1.5 km away has no marker over the world and keeps its mark on the compass, while the pin
//! the player follows, as far, keeps both.

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::rig::{app, ground};
use crate::menus::hud::compass::Pin;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::ui::theme;
use crate::universe::Universe;

/// Returns a pin of `kind` of the player called `name` at the ground point `at` of the home
/// planet.
fn pin(app: &App, kind: MarkerKind, name: &str, at: DVec3) -> Marker {
    Marker {
        kind,
        place: MarkerPlace::Body(app.world().resource::<Universe>().home(), at),
        color: theme::PIN,
        label: name.to_string(),
        owner: 0,
    }
}

/// Returns how many marks of `kind` the compass shows.
fn on_compass(app: &mut App, kind: MarkerKind) -> usize {
    let mut pins = app.world_mut().query::<(&Pin, &Node)>();
    pins.iter(app.world())
        .filter(|(pin, node)| pin.kind == Some(kind) && node.display == Display::Flex)
        .count()
}

/// Returns whether any text of the interface holds `words`.
fn written(app: &mut App, words: &str) -> bool {
    let mut texts = app.world_mut().query::<&Text>();
    texts.iter(app.world()).any(|text| text.0.contains(words))
}

/// On foot, facing north: the followed pin 1.5 km ahead says its name over the world and is on
/// the compass; another pin 1.5 km away, 30 degrees aside, is on the compass and has no marker
/// over the world; a pin 600 m away square to the side, within the limit, has its marker over
/// the world on the border of the screen.
#[test]
fn a_pin_past_the_far_limit_keeps_only_its_compass_mark() {
    let mut app = app("far");
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let marks = vec![
        pin(
            &app,
            MarkerKind::Waypoint,
            "Pin-followed",
            ground(1500.0, 0.0),
        ),
        pin(
            &app,
            MarkerKind::Favourite,
            "Pin-far",
            ground(1300.0, 750.0),
        ),
        pin(&app, MarkerKind::Favourite, "Pin-near", ground(0.0, -600.0)),
    ];
    app.world_mut().resource_mut::<Markers>().0.extend(marks);
    for _ in 0..4 {
        app.update();
    }
    assert!(
        written(&mut app, "Pin-near"),
        "the markers over the world ran"
    );
    assert!(written(&mut app, "Pin-followed 1.5 km"));
    assert!(!written(&mut app, "Pin-far"));
    assert_eq!(on_compass(&mut app, MarkerKind::Waypoint), 1);
    assert_eq!(on_compass(&mut app, MarkerKind::Favourite), 1);
}
