//! Map tests: a right click on bare ground of the flat map drops a pin, and the marker a click
//! selects is the pin whose card shows.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::{drop_click, pick};
use crate::map::level::Level;
use crate::map::pins::{Pin, Pins};
use crate::map::places::Places;
use crate::map::surface::Surface;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::net::SessionNotice;
use crate::universe::BodyId;

/// Two markers on the flat map.
fn pins() -> Pins {
    Pins(vec![
        Pin {
            at: DVec2::new(0.3, 0.1),
            marker: Some(0),
        },
        Pin {
            at: DVec2::new(-0.2, 0.0),
            marker: Some(1),
        },
    ])
}

/// Returns a headless app on the flat map of the default body with the pointer on bare ground,
/// and no pin, after the systems of the pins of the map.
fn ground() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(Level::Planet(BodyId::default()))
        .insert_resource(Surface {
            over: true,
            point: Some(DVec3::X),
            ..default()
        })
        .insert_resource(pins())
        .insert_resource(Markers::default())
        .init_resource::<Places>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_message::<SessionNotice>()
        .add_systems(Update, (drop_click, pick).chain());
    app
}

/// Presses the right button for one frame.
fn right_click(app: &mut App) {
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.clear();
    buttons.press(MouseButton::Right);
    app.update();
}

/// A right click on bare ground of the flat map drops a pin there, none on a marker, and none
/// off the map.
#[test]
fn a_right_click_on_the_ground_drops_a_pin() {
    let mut app = ground();
    right_click(&mut app);
    let places = app.world().resource::<Places>();
    assert_eq!(places.pins.len(), 1);
    assert_eq!(places.pins[0].dir, DVec3::X.to_array());
    app.world_mut().resource_mut::<Surface>().hover = Some(0);
    right_click(&mut app);
    app.world_mut().resource_mut::<Surface>().hover = None;
    app.world_mut().resource_mut::<Surface>().over = false;
    right_click(&mut app);
    assert_eq!(app.world().resource::<Places>().pins.len(), 1);
}

/// The marker a click selects is the pin whose card shows; a marker of a friend is no pin, and
/// letting go of the selection lets go of the pin. A pin picked in the list stays picked.
#[test]
fn the_marker_selected_is_the_selected_pin() {
    let mut app = ground();
    let body = BodyId::default();
    app.world_mut()
        .resource_mut::<Places>()
        .drop(body, DVec3::X);
    let marker = |owner: u32, dir: DVec3| Marker {
        kind: MarkerKind::Favourite,
        place: MarkerPlace::Body(body, dir * 1000.0),
        color: Color::WHITE,
        label: String::new(),
        owner,
    };
    app.world_mut().resource_mut::<Markers>().0 = vec![marker(0, DVec3::X), marker(5, DVec3::X)];
    app.world_mut().resource_mut::<Surface>().chosen = Some(0);
    app.update();
    assert_eq!(app.world().resource::<Places>().selected, Some(0));
    app.world_mut().resource_mut::<Surface>().chosen = Some(1);
    app.update();
    assert_eq!(app.world().resource::<Places>().selected, None);
    app.world_mut().resource_mut::<Places>().selected = Some(0);
    app.update();
    assert_eq!(app.world().resource::<Places>().selected, Some(0));
    app.world_mut().resource_mut::<Surface>().chosen = None;
    app.update();
    assert_eq!(app.world().resource::<Places>().selected, None);
}
