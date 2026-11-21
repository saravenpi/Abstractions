//! Tests of the kit a robot wakes with: a new pack gets a spear and a knife of the stone underfoot
//! with a weapon in the first slot, held; a pack of an old world without tools gets it once; a pack
//! that has tools is left alone.

use super::*;
use crate::craft::build::Intent;
use crate::craft::matter::{Mineral, Stuff};
use crate::universe::{PlanetSpec, Universe};

/// Returns the home planet of universe `seed`.
fn home(seed: u64) -> PlanetSpec {
    let universe = Universe::new(seed);
    universe.planet(universe.home())
}

/// A new pack on the home of the rig gets a spear and a knife knapped from the stone underfoot, each
/// with its mass and unworn, the spear in the first slot and held, and takes no stone from the pack.
#[test]
fn a_new_pack_gets_a_spear_and_a_knife() {
    let planet = home(3);
    let mut pack = Pack::default();
    assert_eq!(pack.equip(&planet.record, planet.gravity), 2);
    assert!(pack.kitted && pack.stacks.is_empty());
    let intents: Vec<Intent> = pack.items.iter().map(Item::intent).collect();
    assert_eq!(intents, vec![Intent::Throw, Intent::Cut]);
    let held = pack.held().expect("the first slot holds a tool");
    assert_eq!(held.intent(), Intent::Throw);
    assert_eq!(pack.bar.held, 0);
    for item in &pack.items {
        let stone = item
            .recipe
            .stuff
            .iter()
            .all(|s| matches!(s, Stuff::Mineral(_)));
        assert!(stone, "{:?} is not all stone", item.recipe.stuff);
        assert!(
            item.mass() > 0.05 && item.mass() < 5.0,
            "{} kg",
            item.mass()
        );
        assert!(item.recipe.wear == Default::default());
    }
    println!("kit: {:.2} kg in all", pack.mass());
}

/// A world saved before kits were given reads its pack as not kitted. Without a tool it gets the kit
/// on its next load, and only once.
#[test]
fn an_old_pack_without_tools_gets_the_kit_once() {
    let planet = home(3);
    let old: Pack = toml::from_str("next = 4\n").expect("an old pack reads");
    assert!(!old.kitted && !old.armed());
    let mut pack = old.clone();
    assert_eq!(pack.equip(&planet.record, planet.gravity), 2);
    assert_eq!(pack.equip(&planet.record, planet.gravity), 0);
    assert_eq!(pack.items.len(), 2);
    let text = toml::to_string(&pack).expect("the pack writes");
    let read: Pack = toml::from_str(&text).expect("the pack reads back");
    assert!(read.kitted && read == pack);
}

/// An old pack that holds a tool already gets no kit, and is kitted for good.
#[test]
fn a_pack_with_a_tool_gets_no_kit() {
    let planet = home(3);
    let mut pack = Pack::default();
    pack.stow(
        Stack {
            stuff: Stuff::Mineral(Mineral::Basalt),
            material: Mineral::Basalt.material(),
            mass: 1.0,
        },
        f32::INFINITY,
    );
    let mut armed = Pack::default();
    armed.equip(&planet.record, planet.gravity);
    let tool = armed.items[0].clone();
    pack.keep(tool.recipe, tool.materials);
    assert_eq!(pack.equip(&planet.record, planet.gravity), 0);
    assert!(pack.kitted && pack.items.len() == 1);
}
