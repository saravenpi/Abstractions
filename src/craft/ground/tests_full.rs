//! Tests of a pack with little room: it refuses a tool it cannot lift with a line that says so,
//! takes what fits of a heap and leaves the rest, and a thing another player took is refused too.

use abstractions_net::wire::Lying;
use bevy::math::DVec3;

use super::tests_round::worn_pack;
use super::{Ground, Shed, TAKEN, lying, shed_item, shed_stack};
use crate::craft::stage::earth;
use crate::universe::BodyId;

/// Returns a ground alone with the worn tool of the test pack and its bone put down, the pack, and
/// the records of the tool and the heap as they lie.
fn laid() -> (Ground, crate::craft::Pack, Lying, Lying) {
    let (mut pack, id) = worn_pack();
    let mut shed = Shed::default();
    shed_item(&mut pack, &mut shed, id);
    shed_stack(&mut pack, &mut shed, 0, 1.0);
    let mut ground = Ground::default();
    for (n, thing) in std::mem::take(&mut shed.0).into_iter().enumerate() {
        let at = (
            BodyId::default(),
            DVec3::new(n as f64 * 10.0, 6_400_000.0, 0.0),
        );
        let put = lying(&thing, at, &earth(), 0.0).expect("a record");
        ground.put(put, thing, &mut pack);
    }
    let [tool, heap] = [0, 1].map(|n| ground.pile.lying[n].clone());
    (ground, pack, tool, heap)
}

/// A tool heavier than the room left stays on the ground, and the line names it, its mass and the
/// room; nothing enters the pack.
#[test]
fn a_full_pack_refuses_a_tool_with_a_line() {
    let (mut ground, mut pack, tool, _) = laid();
    let (items, mut shed) = (pack.items.len(), Shed::default());
    let room = tool.mass * 0.5;
    let line = ground
        .reach(tool.id, room, (&mut pack, &mut shed))
        .expect_err("refused");
    assert!(line.starts_with("Pack full: the "), "{line}");
    assert!(line.contains(" kg") || line.contains(" g"), "{line}");
    assert_eq!((pack.items.len(), ground.pile.lying.len()), (items, 2));
    assert!(ground.asks.is_empty() && shed.0.is_empty());
}

/// A heap gives what the room holds and keeps the rest on the ground; with no room at all it is
/// refused with the same line.
#[test]
fn a_heap_gives_what_fits() {
    let (mut ground, mut pack, _, heap) = laid();
    let mut shed = Shed::default();
    ground
        .reach(heap.id, 1.0, (&mut pack, &mut shed))
        .expect("a part");
    let left = ground.pile.get(heap.id).expect("the rest lies");
    assert!((left.mass - (heap.mass - 1.0)).abs() < 1e-5);
    assert!((pack.stacks[pack.stacks.len() - 1].mass - 1.0).abs() < 1e-5);
    let full = ground.reach(heap.id, 0.0, (&mut pack, &mut shed));
    assert!(full.is_err_and(|line| line.starts_with("Pack full")));
}

/// A thing another player took first is gone from the ground, its take is refused with the line
/// that says so, and a second ask is not sent while the first waits.
#[test]
fn a_thing_taken_first_by_another_is_refused() {
    let (mut ground, mut pack, tool, heap) = laid();
    ground.remote = true;
    let mut shed = Shed::default();
    ground
        .reach(tool.id, 25.0, (&mut pack, &mut shed))
        .expect("asked");
    ground
        .reach(heap.id, 25.0, (&mut pack, &mut shed))
        .expect("waits");
    assert_eq!((ground.asks.len(), ground.asked), (1, Some(tool.id)));
    assert_eq!(ground.denied(tool.id), TAKEN);
    assert_eq!((ground.asked, ground.pile.lying.len()), (None, 1));
    let again = ground.reach(tool.id, 25.0, (&mut pack, &mut shed));
    assert_eq!(again, Err(TAKEN.to_string()));
}
