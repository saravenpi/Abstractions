//! Tests of the drop of the pack: X never deletes a tool, it puts it down on the ground with its
//! wear, and Shift X puts half a stack down.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::Workshop;
use super::bench::Bench;
use super::click::Press;
use super::deeds::Deed;
use super::lines::What;
use crate::craft::build::{Intent, solve};
use crate::craft::ground::{Shed, Thing};
use crate::craft::pack::Pack;
use crate::craft::stage::{earth, pack};

/// Returns an app with the screen state, `carried` as the pack and Shift held when `shift`.
fn app(carried: Pack, shift: bool) -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .init_state::<Workshop>()
        .insert_resource(carried)
        .init_resource::<Bench>()
        .init_resource::<Shed>()
        .init_resource::<ButtonInput<KeyCode>>();
    if shift {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::ShiftLeft);
    }
    app
}

/// Puts the cursor of the pack on the row of `what` and drops it, as X does.
fn drop_row(app: &mut App, what: What) {
    let ran = app.world_mut().run_system_once(move |mut deed: Deed| {
        let at = deed.lines().iter().position(|line| line.what == what);
        deed.bench.cursor = at.expect("listed");
        deed.press(Press::Drop);
    });
    ran.expect("the deed ran");
}

/// X on a tool never deletes it: it leaves the pack and its slot and waits to be put down on the
/// ground, the same tool with the same wear.
#[test]
fn a_dropped_tool_goes_to_the_ground() {
    let mut carried = pack();
    let stock = carried.stock();
    let proposals = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = carried.make(&proposals.lightest, &stock).expect("made");
    let tool = carried.item(id).cloned().expect("kept");
    let mut app = app(carried, false);
    drop_row(&mut app, What::Item(id));
    let left = app.world().resource::<Pack>();
    assert!(left.item(id).is_none() && !left.bar.slots.contains(&Some(id)));
    let shed = &app.world().resource::<Shed>().0;
    assert_eq!(shed.as_slice(), [Thing::Tool(tool)]);
}

/// Shift X puts half of a stack down and keeps the other half in the pack.
#[test]
fn shift_drops_half_a_stack() {
    let carried = pack();
    let (stuff, mass) = (carried.stacks[0].stuff, carried.stacks[0].mass);
    let mut app = app(carried, true);
    drop_row(&mut app, What::Stack(0));
    let kept = app
        .world()
        .resource::<Pack>()
        .stacks
        .iter()
        .find(|s| s.stuff == stuff);
    assert!(kept.is_some_and(|s| (s.mass - mass * 0.5).abs() < 1e-5));
    let [Thing::Heap(down)] = app.world().resource::<Shed>().0.as_slice() else {
        panic!("one heap down");
    };
    assert!((down.mass - mass * 0.5).abs() < 1e-5);
}
