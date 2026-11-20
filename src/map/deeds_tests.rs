//! Map tests: the buttons under the card follow the row under the cursor and the course.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::deeds::{DeedCourse, Reason, deeds, write};
use super::{Deed, offer};
use crate::computer::{Act, Block, Board, Destination, Line, RouteStatus, Target};
use crate::universe::Universe;

/// Returns the acts on the course and the places the course buttons set, and the reason.
fn shown(app: &mut App) -> (Vec<Act>, Vec<Target>, String) {
    app.world_mut().run_system_once(write).unwrap();
    let world = app.world_mut();
    let acts = world.query::<&Act>().iter(world).copied().collect();
    let choices = world
        .query::<&DeedCourse>()
        .iter(world)
        .map(|deed| deed.0)
        .collect();
    let reason = world
        .query_filtered::<&Text, With<Reason>>()
        .single(world)
        .unwrap();
    (acts, choices, reason.0.clone())
}

/// Returns an app with an empty table, no course and an idle route, and the card's buttons.
fn rig() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Board>()
        .init_resource::<Destination>()
        .init_resource::<RouteStatus>();
    app.world_mut().spawn(deeds());
    app
}

/// Returns the row of the planet `target` in a table.
fn row(target: Target) -> Line {
    Line {
        title: "Kolva".to_string(),
        detail: String::new(),
        target,
        distance: 0.0,
        visited: true,
        at: default(),
    }
}

/// Without a row the card has no button. On a planet it offers to set the course, and once the
/// planet is the course it offers to engage while the route is idle and to clear it, with the
/// reason the drive waits while it is blocked.
#[test]
fn the_buttons_follow_the_cursor_and_the_course() {
    let planet = Target::Body(Universe::new(7).home());
    let mut app = rig();
    assert_eq!(shown(&mut app), (vec![], vec![], String::new()));
    let mut board = app.world_mut().resource_mut::<Board>();
    board.fill("Varena system".to_string(), vec![row(planet)]);
    assert_eq!(shown(&mut app), (vec![], vec![planet], String::new()));
    app.world_mut().resource_mut::<Destination>().0 = Some(planet);
    let (acts, courses, _) = shown(&mut app);
    assert_eq!((acts, courses), (vec![Act::Engage, Act::Clear], vec![]));
    *app.world_mut().resource_mut::<RouteStatus>() = RouteStatus::Blocked(Block::Landed);
    let (acts, _, reason) = shown(&mut app);
    assert_eq!(
        (acts, reason.as_str()),
        (vec![Act::Clear], "Landed: take off first")
    );
    let free = offer(planet, None, RouteStatus::Idle);
    assert_eq!(free[0], Deed::Course);
}
