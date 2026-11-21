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

/// Returns the test pack with one tool made and lent out, the id of the tool and what was lent.
fn lent_out() -> (crate::craft::Pack, u32, crate::craft::Lent) {
    let mut pack = pack();
    let stock = pack.stock();
    let proposals = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = pack.make(&proposals.lightest, &stock).expect("made");
    let lent = pack.lend(id).expect("the tool is there");
    (pack, id, lent)
}

/// A lent tool is noted as out in the world with the place it rests at, a place within the slack
/// is not noted again, and the note goes when the tool comes back.
#[test]
fn a_tool_that_is_out_is_noted_where_it_rests() {
    use crate::universe::BodyId;
    use bevy::math::DVec3;
    let (mut pack, id, lent) = lent_out();
    assert_eq!(pack.out.len(), 1);
    assert_eq!((pack.out[0].item.id, pack.out[0].body), (id, None));
    let (body, at) = (BodyId::default(), DVec3::new(10.0, 2.0, 3.0));
    assert!(pack.rest(id, (body, at), 0.5));
    assert!(!pack.rest(id, (body, at + DVec3::X * 0.2), 0.5));
    assert!(pack.rest(id, (body, at + DVec3::X), 0.5));
    assert_eq!(pack.out[0].at, [11.0, 2.0, 3.0]);
    assert!(!pack.rest(id + 1, (body, at), 0.5));
    pack.restore(lent);
    assert!(pack.out.is_empty() && pack.item(id).is_some());
}

/// A tool that is out is kept in the saved pack, in flight and at rest, and a robot that falls
/// keeps it: it is not in the cache and not forgotten, and no new tool takes its id.
#[test]
fn a_tool_that_is_out_survives_a_save_and_a_fall() {
    use crate::craft::fall::{Left, leave};
    use crate::universe::BodyId;
    use bevy::math::DVec3;
    let (mut pack, id, _) = lent_out();
    let read = |pack: &crate::craft::Pack| {
        let saved = toml::to_string(pack).expect("the pack is saved");
        toml::from_str::<crate::craft::Pack>(&saved).expect("the pack is read")
    };
    assert_eq!(read(&pack), pack);
    assert!(pack.rest(id, (BodyId::default(), DVec3::ONE), 0.5));
    assert_eq!(read(&pack), pack);
    let mut left = Left::default();
    leave(&mut left, &mut pack, (BodyId::default(), DVec3::ZERO));
    assert_eq!(pack.out.len(), 1, "the fall lost the tool that was out");
    assert!(left.0.expect("a cache").pack.out.is_empty());
    assert!(
        pack.next > id,
        "a new tool would take the id of the one out"
    );
}
