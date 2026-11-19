//! Tests of the favourites: drawn once the universe exists.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Favourites;
use crate::menus::{MarkerKind, Markers};
use crate::universe::Universe;

/// Favourites set before the universe exists are drawn once it does.
#[test]
fn favourites_set_before_the_universe_are_drawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Markers>()
        .init_resource::<Favourites>()
        .add_systems(Update, super::fill);
    app.update();
    let home = Universe::new(3).home();
    app.world_mut()
        .resource_mut::<Favourites>()
        .save(home, DVec3::X);
    app.update();
    assert!(app.world().resource::<Markers>().0.is_empty());
    app.world_mut().insert_resource(Universe::new(3));
    app.update();
    let markers = &app.world().resource::<Markers>().0;
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].kind, MarkerKind::Favourite);
}

