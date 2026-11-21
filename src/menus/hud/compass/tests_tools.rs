//! Tests of the tool markers: a tool out in the world is marked where it came to rest, named for the
//! tool, and its marker goes once the tool is back in the pack.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::tools::{mark, tools};
use crate::craft::Pack;
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::universe::{BodyId, Universe};

/// Returns a pack with the kit of the home of universe 3.
fn kitted() -> Pack {
    let universe = Universe::new(3);
    let planet = universe.planet(universe.home());
    let mut pack = Pack::default();
    pack.equip(&planet.record, planet.gravity);
    pack
}

/// A thrown spear is marked once it came to rest, where it rests, as "Your" and the name of the
/// tool; it is not marked while it still flies, and its marker goes once it is picked up.
#[test]
fn a_tool_out_in_the_world_is_marked_until_it_is_back() {
    let mut pack = kitted();
    let id = pack.items[0].id;
    let lent = pack.lend(id).expect("the spear leaves the pack");
    assert!(mark(&pack.out[0]).is_none());
    let at = DVec3::new(1.0, 2.0, 3.0);
    pack.rest(id, (BodyId::default(), at), 0.5);
    let marker = mark(&pack.out[0]).expect("a marker once the spear rests");
    assert_eq!(marker.kind, MarkerKind::Tool);
    assert_eq!(marker.place, MarkerPlace::Body(BodyId::default(), at));
    assert!(
        marker.label.starts_with("Your ") && marker.label.contains("javelin"),
        "{}",
        marker.label
    );
    let mut app = App::new();
    app.init_resource::<Markers>().insert_resource(pack);
    app.world_mut()
        .run_system_once(tools)
        .expect("the markers run");
    let count = |app: &App| app.world().resource::<Markers>().0.len();
    assert_eq!(count(&app), 1);
    app.world_mut().resource_mut::<Pack>().restore(lent);
    app.world_mut()
        .run_system_once(tools)
        .expect("the markers run");
    assert_eq!(count(&app), 0);
}
