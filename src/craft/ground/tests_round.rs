//! Tests of the ground alone: a tool or a heap put down and taken back keeps what it is, heaps of
//! one material put down together merge, and the ground survives the world file.

use abstractions_net::wire::{Act, Lying};
use bevy::math::DVec3;

use super::{Ground, Shed, Thing, lying, shed_item, shed_stack};
use crate::craft::build::{Intent, solve};
use crate::craft::matter::{Mineral, Stuff};
use crate::craft::pack::Pack;
use crate::craft::stage::{earth, pack};
use crate::saves::WorldSave;
use crate::universe::BodyId;

/// Returns where things are put down in these tests: on the default body, `x` metres along it.
fn spot(x: f64) -> (BodyId, DVec3) {
    (BodyId::default(), DVec3::new(x, 6_400_000.0, 0.0))
}

/// Returns the record of `thing` put down at `x` metres.
fn record(thing: &Thing, x: f64) -> Lying {
    lying(thing, spot(x), &earth(), 50.0).expect("a record")
}

/// Returns the test pack with a cutting tool made and worn by a few blows, and its id.
pub(super) fn worn_pack() -> (Pack, u32) {
    let mut pack = pack();
    let stock = pack.stock();
    let proposals = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = pack.make(&proposals.lightest, &stock).expect("made");
    let at = pack
        .items
        .iter()
        .position(|item| item.id == id)
        .expect("kept");
    pack.items[at].recipe.wear.0 = [3, 1, 0, 2];
    (pack, id)
}

/// A worn tool dropped from the pack lies on the ground with its mass and wear, and comes back into
/// a pack with the same recipe, wear and materials; the pack weighs what it weighed.
#[test]
fn a_tool_comes_back_as_it_went() {
    let (mut pack, id) = worn_pack();
    let (before, item) = (pack.mass(), pack.item(id).cloned().expect("the tool"));
    let mut shed = Shed::default();
    shed_item(&mut pack, &mut shed, id);
    assert!(pack.item(id).is_none() && !pack.bar.slots.contains(&Some(id)));
    let thing = shed.0.pop().expect("shed");
    let mut ground = Ground::default();
    assert_eq!(ground.put(record(&thing, 0.0), thing, &mut pack), None);
    let lies = ground.pile.lying[0].clone();
    assert!((lies.mass - item.mass()).abs() < 1e-6);
    assert_eq!(lies.wear, item.worn(&earth()).0);
    ground
        .reach(lies.id, 25.0, (&mut pack, &mut shed))
        .expect("taken");
    assert!(ground.pile.lying.is_empty() && shed.0.is_empty());
    let back = pack.items.last().expect("back in the pack");
    assert_eq!(
        (&back.recipe, &back.materials),
        (&item.recipe, &item.materials)
    );
    assert!((pack.mass() - before).abs() < 1e-5);
}

/// Half a stack of flint dropped twice at the same spot lies as one heap of both halves, and taken
/// back fills the stack it came from.
#[test]
fn heaps_merge_and_come_back() {
    let mut pack = pack();
    let flint = Stuff::Mineral(Mineral::Flint);
    let of = |pack: &Pack| pack.stacks.iter().position(|s| s.stuff == flint);
    let mut shed = Shed::default();
    let at = of(&pack).expect("flint");
    shed_stack(&mut pack, &mut shed, at, 0.5);
    let at = of(&pack).expect("half left");
    shed_stack(&mut pack, &mut shed, at, 1.0);
    assert!(of(&pack).is_none());
    let mut ground = Ground::default();
    for (n, thing) in std::mem::take(&mut shed.0).into_iter().enumerate() {
        let put = record(&thing, n as f64 * 0.5);
        assert_eq!(ground.put(put, thing, &mut pack), None);
    }
    let [heap] = ground.pile.lying.as_slice() else {
        panic!("one heap: {:?}", ground.pile.lying);
    };
    let (id, mass) = (heap.id, heap.mass);
    assert!((mass - 1.5).abs() < 1e-5);
    ground
        .reach(id, 25.0, (&mut pack, &mut shed))
        .expect("taken");
    let back = pack.stacks[of(&pack).expect("flint back")];
    assert!((back.mass - 1.5).abs() < 1e-5);
}

/// What lies on the ground is written in the world file and read back as it was.
#[test]
fn the_ground_survives_the_world_file() {
    let (mut pack, id) = worn_pack();
    let mut shed = Shed::default();
    shed_item(&mut pack, &mut shed, id);
    shed_stack(&mut pack, &mut shed, 0, 1.0);
    let mut ground = Ground::default();
    for (n, thing) in std::mem::take(&mut shed.0).into_iter().enumerate() {
        ground.put(record(&thing, n as f64 * 5.0), thing, &mut pack);
    }
    let save = WorldSave {
        ground: ground.pile.clone(),
        ..WorldSave::default()
    };
    let text = toml::to_string(&save).expect("written");
    let back: WorldSave = toml::from_str(&text).expect("read");
    assert_eq!(back.ground, ground.pile);
    assert_eq!(back.ground.lying.len(), 2);
}

/// In a session a drop is asked of the server under a tag and waits; the answer that it lies clears
/// it, and a drop the server hands back returns to the pack with the line that says why.
#[test]
fn a_drop_in_a_session_waits_for_the_server() {
    let (mut pack, id) = worn_pack();
    let mut shed = Shed::default();
    shed_item(&mut pack, &mut shed, id);
    shed_stack(&mut pack, &mut shed, 0, 1.0);
    let mut ground = Ground {
        remote: true,
        ..Ground::default()
    };
    let things: Vec<Thing> = std::mem::take(&mut shed.0);
    let records: Vec<Lying> = things.iter().map(|t| record(t, 0.0)).collect();
    for (put, thing) in records.into_iter().zip(things) {
        assert_eq!(ground.put(put, thing, &mut pack), None);
    }
    assert!(ground.pile.lying.is_empty());
    let [Act::Drop(first), Act::Drop(second)] = ground.asks.as_slice() else {
        panic!("two drops asked: {:?}", ground.asks);
    };
    assert_eq!((first.tag, second.tag, ground.pending.len()), (1, 2, 2));
    let me = Some(first.by);
    let laid = Lying {
        id: 4,
        ..first.clone()
    };
    let refused = second.tag;
    ground.heard(laid, me);
    assert_eq!((ground.pending.len(), ground.pile.lying.len()), (1, 1));
    let stacks = pack.stacks.len();
    assert_eq!(ground.kept(refused, &mut pack), super::NO_ROOM);
    assert!(ground.pending.is_empty() && pack.stacks.len() == stacks + 1);
}
