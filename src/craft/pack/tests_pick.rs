//! Tests of picking stones up: a rock under the crosshair gives a piece of the stone it looks like, at
//! most a handful and at most the room, and keeps the rest. Nothing under the crosshair gives nothing.

use bevy::prelude::*;

use super::Pack;
use super::pick::{Knocked, chips, piece, stone_of, take};
use super::tune::HANDFUL;
use crate::craft::matter::{Mineral, Stuff};
use crate::world::rocks::{Plant, Rock, RockId, Taken};

/// Returns a rock with the id `id` drawn at `scale`.
fn rock(id: u64, scale: f32) -> Rock {
    Rock {
        id: RockId(id),
        plant: Plant::Rock,
        scale,
        at: Vec3::ZERO,
        mid: Vec3::Y * 0.45,
        left: 1.0,
    }
}

/// With no rock under the ray, nothing is stowed and nothing is taken.
#[test]
fn no_rock_under_the_ray_stows_nothing() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let hit = take(None, 25.0, (&mut pack, &mut taken));
    assert_eq!(hit, None);
    assert!(pack.stacks.is_empty());
    assert_eq!(taken, Taken::default());
}

/// A rock gives the stone it looks like, whatever its id: basalt columns and the basalt block of the
/// deep floor give basalt, a flint nodule flint, obsidian shards obsidian, an ice block ice, and the
/// stone that lands in the pack is that one. Sandstone crumbles and gives nothing.
#[test]
fn a_rock_gives_the_stone_it_looks_like() {
    let looks = [
        (Plant::Rock, Some(Mineral::Basalt)),
        (Plant::Boulder, Some(Mineral::Basalt)),
        (Plant::Flint, Some(Mineral::Flint)),
        (Plant::Obsidian, Some(Mineral::Obsidian)),
        (Plant::IceRock, Some(Mineral::Ice)),
        (Plant::Quartz, Some(Mineral::Quartz)),
        (Plant::Jade, Some(Mineral::Jade)),
        (Plant::Ironstone, Some(Mineral::Ironstone)),
        (Plant::Malachite, Some(Mineral::Malachite)),
        (Plant::Brimstone, Some(Mineral::Brimstone)),
        (Plant::Siderite, Some(Mineral::Siderite)),
        (Plant::Sandstone, None),
    ];
    for (plant, want) in looks {
        for n in 0..4 {
            let rock = Rock {
                plant,
                ..rock(n, 1.0)
            };
            assert_eq!(stone_of(&rock), want, "{plant:?}");
            let (mut pack, mut taken) = (Pack::default(), Taken::default());
            let hit = take(Some(rock), 25.0, (&mut pack, &mut taken));
            assert_eq!(hit.is_some(), want.is_some(), "{plant:?}");
            let got = pack.stacks.first().map(|stack| stack.stuff);
            assert_eq!(got, want.map(Stuff::Mineral), "{plant:?}");
        }
    }
}

/// A pick takes at most a handful, and the rock keeps the rest: its taken mass grows by the piece.
#[test]
fn a_pick_takes_at_most_a_handful_and_the_rock_keeps_the_rest() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let big = rock(4, 1.0);
    let hit = take(Some(big), 25.0, (&mut pack, &mut taken));
    assert_eq!(hit, Some((RockId(4), HANDFUL)));
    assert!((pack.mass() - HANDFUL).abs() < 1.0e-4);
    assert!((taken.kilograms(RockId(4)) - HANDFUL).abs() < 1.0e-4);
    assert!(taken.left(RockId(4)) < 1.0 && taken.left(RockId(4)) > 0.5);
    take(Some(big), 25.0, (&mut pack, &mut taken));
    assert!((taken.kilograms(RockId(4)) - 2.0 * HANDFUL).abs() < 1.0e-4);
}

/// A pick takes no more than the room left in the pack, and nothing when there is none.
#[test]
fn a_pick_takes_at_most_the_room_and_nothing_without_it() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    take(Some(rock(4, 1.0)), 1.0, (&mut pack, &mut taken));
    assert!((pack.mass() - 1.0).abs() < 1.0e-4);
    let (mut full, mut kept) = (Pack::default(), Taken::default());
    let hit = take(Some(rock(4, 1.0)), 0.0, (&mut full, &mut kept));
    assert_eq!(hit, None);
    assert!(full.stacks.is_empty());
    assert_eq!(kept, Taken::default());
}

/// A rock small enough to carry goes whole, and is gone: it weighs its volume times its density.
#[test]
fn a_small_rock_goes_whole_and_is_gone() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let small = rock(4, 0.08);
    let hit = take(Some(small), 25.0, (&mut pack, &mut taken));
    let (id, kilograms) = hit.expect("a piece");
    let density = pack.stacks[0].material.density;
    let volume = 2.0 / 3.0 * std::f32::consts::PI * 0.056 * 0.056 * 0.072;
    assert!((kilograms - volume * density).abs() < 1.0e-3);
    assert!(kilograms < HANDFUL);
    assert_eq!(taken.left(id), 0.0);
}

/// A rock is gone only when its mass is used up: a pick at a time, the handful of each adds up to the
/// whole rock and not a gram more.
#[test]
fn picks_add_up_to_the_whole_rock() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let small = rock(4, 0.3);
    let mut picks = 0;
    while taken.left(RockId(4)) > 0.0 && picks < 1000 {
        take(Some(small), 100.0, (&mut pack, &mut taken));
        picks += 1;
    }
    let whole = taken.whole(RockId(4)).expect("a quarry");
    assert!(picks > 1);
    assert!((pack.mass() - whole).abs() < 1.0e-2);
}

/// Breaking a piece off a stone throws five smaller angular chips fanned outward around the normal.
#[test]
fn breaking_a_rock_sprays_five_outward_chips() {
    let knocked = Knocked {
        rock: rock(8, 1.0),
        at: Vec3::new(0.0, 0.5, -1.5),
    };
    let main = piece(Mineral::Quartz, &knocked, (HANDFUL, Vec3::Y));
    let spray = chips(Mineral::Quartz, &knocked, (HANDFUL, Vec3::Y));
    assert_eq!(spray.len(), 5);
    for chip in spray {
        assert!(chip.radius >= main.radius * 0.34 && chip.radius <= main.radius * 0.55);
        assert!(chip.velocity.y > 0.0 && chip.velocity.xz().length() > 0.5);
    }
}

/// Hatchets (`Intent::Cut`) fell trees in two hits with double wood logs, and pickaxes (`Intent::Crush`)
/// boost rock breaking and underground digging depth and ore yield.
#[test]
fn cut_and_crush_tools_boost_chopping_and_mining() {
    use super::pick::chop::{apply_chop, chop_power};
    use super::pick::dig::{dig_power, dug_mineral};
    use super::pick::take_scaled;
    use crate::craft::build::Intent;
    use crate::craft::matter::WoodKind;
    use crate::universe::Universe;
    use crate::world::trees::{Felled, TreeId};
    use bevy::math::DVec3;

    let mut felled = Felled::default();
    let mut pack = Pack::default();
    let bare = chop_power(None);
    let axe = chop_power(Some(Intent::Cut));
    assert_eq!(bare, (1, 1));
    assert_eq!(axe, (2, 2));
    assert!(!apply_chop(TreeId(5), axe, &mut felled, &mut pack).0);
    assert!(apply_chop(TreeId(5), axe, &mut felled, &mut pack).0);
    assert!(pack.count_of(&Stuff::Wood(WoodKind::Log)) >= 6);

    let (bare_d, bare_c) = dig_power(None);
    let (pick_d, pick_c) = dig_power(Some(Intent::Crush));
    assert!(pick_d > bare_d && pick_c > bare_c);
    let u = Universe::new(33);
    let rec = u.planet(u.home()).record;
    let m = dug_mineral(&rec, 33, DVec3::new(0.0, 6_400_000.0, 0.0), 1.6);
    assert!(m.mined_on(&rec, 1.6) || m.lies_on(&rec));

    let (mut p_bare, mut t_bare) = (Pack::default(), Taken::default());
    let (mut p_pick, mut t_pick) = (Pack::default(), Taken::default());
    let (_, kg_bare) =
        take_scaled(Some(rock(12, 1.0)), 100.0, 1.0, (&mut p_bare, &mut t_bare)).unwrap();
    let (_, kg_pick) =
        take_scaled(Some(rock(12, 1.0)), 100.0, 2.5, (&mut p_pick, &mut t_pick)).unwrap();
    assert!(kg_pick > kg_bare * 2.0);
}
