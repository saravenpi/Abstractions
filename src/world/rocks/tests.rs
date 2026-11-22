//! Tests of the rock ids, the taken set and the stand of a chunk.

use super::*;
use crate::saves::WorldSave;
use crate::world::scatter::{Placement, Rig};

/// Returns a prop of kind `plant` standing at `(x, z)`.
fn prop(plant: Plant, x: f32, z: f32) -> Placement {
    Placement {
        pos: Vec3::new(x, 4.0, z),
        yaw: 0.5,
        scale: 1.0,
        plant,
        variant: 1,
        lean: Vec2::ZERO,
    }
}

/// The id of a prop follows its chunk, its kind and its place to half a metre, and nothing else.
#[test]
fn an_id_follows_the_chunk_the_kind_and_the_place() {
    let rock = prop(Plant::Rock, 3.0, 9.0);
    let id = id_of(1, &rock);
    assert_eq!(id, id_of(1, &rock));
    assert_eq!(id, id_of(1, &prop(Plant::Rock, 3.01, 9.0)));
    assert_ne!(id, id_of(2, &rock));
    assert_ne!(id, id_of(1, &prop(Plant::Rock, 4.0, 9.0)));
    assert_ne!(id, id_of(1, &prop(Plant::IceRock, 3.0, 9.0)));
}

/// An id keeps all 64 bits through its text, and the quarried masses come back from a world file
/// written and read again, ids above the signed range included. A row of a plain id, as the first
/// files wrote, loads as a rock taken whole.
#[test]
fn the_taken_map_survives_a_save_round_trip() {
    let id = RockId(u64::MAX - 5);
    assert_eq!(RockId::parse(&id.text()), Some(id));
    assert_eq!(
        Taken::from_text(&["not:an:id".to_string()]),
        Taken::default()
    );
    let mut taken = Taken::default();
    for (n, kg) in [(3, 0.25), (u64::MAX, 12.5), (1 << 63, 2700.0), (42, 0.1)] {
        taken.quarry(RockId(n), kg, 3000.0);
    }
    let save = WorldSave {
        taken: taken.to_text(),
        ..WorldSave::default()
    };
    let text = toml::to_string(&save).expect("a world file");
    let back: WorldSave = toml::from_str(&text).expect("the file again");
    assert_eq!(Taken::from_text(&back.taken), taken);
    let old = Taken::from_text(&[RockId(9).text()]);
    assert_eq!(old.left(RockId(9)), 0.0);
    let file: WorldSave = toml::from_str("name = \"Old\"\nseed = 7\n").expect("an old file");
    assert!(file.taken.is_empty());
}

/// The share left follows the mass taken, a rock nobody touched is whole, and what is left under the
/// least counts as nothing.
#[test]
fn a_rock_is_gone_only_when_its_mass_is_used_up() {
    let (id, mut taken) = (RockId(1), Taken::default());
    assert_eq!(taken.left(id), 1.0);
    taken.quarry(id, 25.0, 100.0);
    assert!((taken.left(id) - 0.75).abs() < 1.0e-6);
    taken.quarry(id, 74.9999, 100.0);
    assert_eq!(taken.left(id), 0.0);
    taken.quarry(id, 10.0, 100.0);
    assert_eq!(taken.kilograms(id), 100.0);
}

/// Spare leaves out the used-up rocks, shrinks the partly taken ones to the cube root of what is left
/// and never touches a tree.
#[test]
fn spare_leaves_out_and_shrinks_the_quarried_rocks() {
    let mut props = vec![
        prop(Plant::Rock, 1.0, 1.0),
        prop(Plant::Flint, 8.0, 1.0),
        prop(Plant::Oak, 1.0, 1.0),
    ];
    let mut taken = Taken::default();
    taken.quarry(id_of(9, &props[0]), 100.0, 100.0);
    taken.quarry(id_of(9, &props[1]), 87.5, 100.0);
    taken.quarry(id_of(9, &props[2]), 100.0, 100.0);
    spare(&mut props, 9, &taken);
    assert_eq!(props.len(), 2);
    assert!((props[0].scale - 0.125_f32.cbrt()).abs() < 1.0e-6);
    assert_eq!((props[1].plant, props[1].scale), (Plant::Oak, 1.0));
}

/// A rock that loses a little keeps its size and its stand stays clean. One that loses over the step
/// shrinks to the cube root of what is left. A used-up rock leaves the stand with its prop, once, and
/// the stand says where each stood.
#[test]
fn a_quarried_rock_shrinks_and_then_leaves_the_stand() {
    let props = [
        prop(Plant::Rock, -10.0, 3.0),
        prop(Plant::Oak, 5.0, 5.0),
        prop(Plant::Flint, 12.0, -8.0),
    ];
    let mut stand = Stand::of(7, &props, &Rig::PLAIN, &Taken::default());
    assert_eq!((stand.rocks().len(), stand.props().len()), (2, 3));
    let first = stand.rocks()[0];
    assert!(stand.quarry(first.id, 0.99));
    assert!(!stand.dirty());
    assert!(stand.quarry(first.id, 0.5));
    assert!(stand.dirty());
    let shrunk = stand.rocks()[0];
    assert!((shrunk.scale - 0.5_f32.cbrt()).abs() < 1.0e-5);
    assert!(
        stand
            .props()
            .iter()
            .any(|p| (p.scale - shrunk.scale).abs() < 1.0e-5)
    );
    assert_eq!(stand.settle().len(), 1);
    assert!(stand.quarry(first.id, 0.0));
    assert_eq!((stand.rocks().len(), stand.props().len()), (1, 2));
    assert_eq!(stand.settle(), vec![(first.at, 0.0)]);
    assert!(!stand.quarry(first.id, 0.0));
    assert!(!stand.dirty());
}

/// A point on the body of a rock finds that rock, a point in the air finds none, a leaning rock tilts its
/// midpoint along the slope normal, and a rock weighs its volume times the density of its stone.
#[test]
fn a_point_on_a_rock_finds_it_and_it_has_a_weight() {
    let props = [
        prop(Plant::Rock, -10.0, 3.0),
        prop(Plant::Oak, 5.0, 5.0),
        prop(Plant::Flint, 12.0, -8.0),
    ];
    let stand = Stand::of(7, &props, &Rig::PLAIN, &Taken::default());
    let rock = stand.rocks()[0];
    let on_it = rock.mid + Vec3::new(0.3, 0.2, 0.0);
    assert_eq!(stand.near(on_it).map(|r| r.id), Some(rock.id));
    assert_eq!(stand.near(rock.mid + Vec3::Y * 20.0), None);
    assert_eq!(stand.near(Vec3::new(5.0, 5.0, 5.0)), None);
    let volume = 2.0 / 3.0 * std::f32::consts::PI * 0.7 * 0.7 * 0.9;
    assert!((rock.mass(3000.0) - volume * 3000.0).abs() < 1.0);
    assert!(rock.mass(917.0) < rock.mass(3000.0));
    let leaning = Placement {
        lean: Vec2::new(0.6, 0.0),
        ..prop(Plant::Rock, 0.0, 0.0)
    };
    let tilted = Stand::of(1, &[leaning], &Rig::PLAIN, &Taken::default()).rocks()[0];
    let want = tilted.at + Vec3::new(-0.6, 1.0, 0.0).normalize() * (0.9 * 0.5);
    assert!(tilted.mid.distance(want) < 1e-5, "{:?}", tilted.mid);
}

/// Temperate worlds keep only basalt, obsidian and flint; other classes and climates add their
/// own minerals, and every mineral maps to its prop and back while sandstone still gives none.
#[test]
fn planetary_minerals_follow_class_and_map_to_props() {
    use crate::universe::{PlanetClass::*, PlanetRecord, Water};

    let t = PlanetRecord::TEMPERATE;
    let temp: Vec<_> = Mineral::ALL.into_iter().filter(|m| m.lies_on(&t)).collect();
    assert_eq!(temp, [Mineral::Basalt, Mineral::Obsidian, Mineral::Flint]);
    let desert = PlanetRecord {
        class: Desert,
        water: Water::None,
        ocean: 0.0,
        ..t
    };
    assert!(Mineral::Quartz.lies_on(&desert) && Mineral::Ironstone.lies_on(&desert));
    assert!(Mineral::Malachite.lies_on(&desert) && !Mineral::Jade.lies_on(&desert));
    let lava = PlanetRecord {
        class: Lava,
        water: Water::None,
        temperature: 1200.0,
        ..t
    };
    assert!(Mineral::Brimstone.lies_on(&lava) && Mineral::Ironstone.lies_on(&lava));
    let barren = PlanetRecord {
        class: Barren,
        water: Water::None,
        pressure: 0.0,
        ..t
    };
    assert!(Mineral::Siderite.lies_on(&barren) && Mineral::Quartz.lies_on(&barren));
    Mineral::ALL
        .into_iter()
        .for_each(|m| assert_eq!(mineral_of(m.plant()), Some(m)));
    assert_eq!(mineral_of(Plant::Sandstone), None);
}
