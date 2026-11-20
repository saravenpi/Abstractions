//! Tests of lending a tool: a thrown tool leaves the pack and its slot, and comes back to the slot.

use crate::craft::build::{Intent, solve};
use crate::craft::stage::{earth, pack};

/// A lent tool is out of the pack and off the hotbar, and restored to the slot it stood in.
#[test]
fn a_lent_tool_comes_back_to_its_slot() {
    let mut pack = pack();
    let stock = pack.stock();
    let proposals = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = pack.make(&proposals.lightest, &stock).expect("made");
    pack.bar.put(3, id);
    let before = pack.mass();
    let lent = pack.lend(id).expect("the tool is there");
    assert!(pack.item(id).is_none() && !pack.bar.slots.contains(&Some(id)));
    assert!(pack.mass() < before);
    assert!(pack.lend(id).is_none());
    pack.restore(lent);
    assert_eq!(pack.bar.slots[3], Some(id));
    assert!((pack.mass() - before).abs() < 1e-6);
}
