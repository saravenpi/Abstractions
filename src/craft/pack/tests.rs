//! Tests of the pack: what the robot carries under gravity, stacks, the hotbar, making a tool and
//! treating a tissue, and the pack surviving the world file.

use super::*;
use crate::craft::build::{Intent, solve};
use crate::craft::matter::{Class, Mineral, Treatment};
use crate::craft::stage::{PLANET, earth, pack, stone, tissue};
use crate::saves::WorldSave;

/// The robot carries 25 kg under 9.81 m/s², less under heavier gravity, and walks slower only
/// past 60 % of that, down to half its speed at a full load.
#[test]
fn the_load_follows_gravity() {
    assert!((capacity(9.81) - 25.0).abs() < 1e-4);
    assert!((capacity(22.0) - 25.0 * 9.81 / 22.0).abs() < 1e-4);
    assert_eq!(pace(10.0, 25.0), 1.0);
    assert_eq!(pace(15.0, 25.0), 1.0);
    assert!(pace(20.0, 25.0) < 1.0 && pace(20.0, 25.0) > 0.5);
    assert!((pace(25.0, 25.0) - 0.5).abs() < 1e-6);
    assert!((Burden::of(20.0, 22.0).room()).abs() < 1e-6);
}

/// The same material stacks, a different grade or stone does not, and stowing stops at the room.
#[test]
fn materials_stack_and_stop_at_the_room() {
    let mut pack = Pack::default();
    assert_eq!(pack.stow(tissue(Class::Bone, 0, 2.0), 10.0), 2.0);
    assert_eq!(pack.stow(tissue(Class::Bone, 0, 2.0), 1.5), 1.5);
    assert_eq!(pack.stacks.len(), 1);
    pack.stow(stone(Mineral::Flint, 1.0), 10.0);
    assert_eq!(pack.stacks.len(), 2);
    assert!((pack.mass() - 4.5).abs() < 1e-5);
    assert!(!pack.take(&stone(Mineral::Flint, 0.0).stuff, 2.0));
    assert!(pack.take(&stone(Mineral::Flint, 0.0).stuff, 1.0));
    assert_eq!(pack.stacks.len(), 1);
}

/// A slot points at one item at most, and the held slot gives the held item.
#[test]
fn the_hotbar_holds_one_item_per_slot() {
    let mut bar = Bar::default();
    bar.place(7);
    bar.place(7);
    bar.place(9);
    assert_eq!(bar.slots[..2], [Some(7), Some(9)]);
    bar.put(4, 7);
    assert_eq!((bar.slots[0], bar.slots[4]), (None, Some(7)));
    bar.select(4);
    assert_eq!(bar.held(), Some(7));
    bar.select(SLOTS);
    assert_eq!(bar.held, 4);
    bar.forget(7);
    assert_eq!(bar.held(), None);
}

/// Making a proposal takes its materials, keeps the tool on the hotbar and weighs what it took.
#[test]
fn making_consumes_the_materials() {
    let mut pack = pack();
    let before = pack.mass();
    let stock = pack.stock();
    let proposals = solve(Intent::Cut, &stock, &earth()).expect("a cut");
    let id = pack.make(&proposals.lightest, &stock).expect("made");
    let item = pack.item(id).expect("kept");
    assert_eq!(pack.bar.slots[0], Some(id));
    assert!(
        (pack.mass() - before).abs() < 0.05,
        "{} {before}",
        pack.mass()
    );
    assert!(pack.stacks.iter().map(|s| s.mass).sum::<f32>() < before - item.mass() + 0.05);
    assert!(item.build().is_some() && !item.name().is_empty());
    let mut poor = Pack::default();
    assert!(poor.make(&proposals.lightest, &stock).is_none());
}

/// Treating a tissue moves its whole stack to the treated stuff, once; a stone takes nothing.
#[test]
fn treating_moves_the_stack() {
    let mut pack = pack();
    let wood = pack.stacks[1].stuff;
    assert!(pack.treat(&wood, Treatment::SunDry));
    assert!(!pack.stacks.iter().any(|s| s.stuff == wood));
    let dried = pack.stacks.last().expect("dried").stuff;
    assert!(!pack.treat(&dried, Treatment::SunDry));
    assert!(!pack.treat(&stone(Mineral::Flint, 0.0).stuff, Treatment::SunDry));
}

/// The pack, a planet seed past i64::MAX included, survives the TOML world file.
#[test]
fn the_pack_survives_the_world_file() {
    let mut save = WorldSave {
        pack: pack(),
        ..WorldSave::default()
    };
    let stock = save.pack.stock();
    let made = solve(Intent::Crush, &stock, &earth()).expect("a crush");
    save.pack.make(&made.durable, &stock);
    let text = toml::to_string(&save).expect("written");
    assert!(text.contains(&PLANET.to_string()));
    let back: WorldSave = toml::from_str(&text).expect("read");
    assert_eq!(back.pack, save.pack);
}
