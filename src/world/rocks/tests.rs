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
    }
}

/// Returns the stand of a chunk with two rocks and a tree.
fn stand() -> Stand {
    let props = [
        prop(Plant::Rock, -10.0, 3.0),
        prop(Plant::Oak, 5.0, 5.0),
        prop(Plant::Sandstone, 12.0, -8.0),
    ];
    Stand::of(7, &props, &Rig::PLAIN)
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

/// An id keeps all 64 bits through its text, and the taken rocks come back from a world file written
/// and read again, ids above the signed range included.
#[test]
fn the_taken_set_survives_a_save_round_trip() {
    let id = RockId(u64::MAX - 5);
    assert_eq!(RockId::parse(&id.text()), Some(id));
    assert_eq!(
        Taken::from_text(&["not an id".to_string()]),
        Taken::default()
    );
    let mut taken = Taken::default();
    for n in [3, u64::MAX, 1 << 63, 42] {
        taken.add(RockId(n));
    }
    let save = WorldSave {
        taken: taken.to_text(),
        ..WorldSave::default()
    };
    let text = toml::to_string(&save).expect("a world file");
    let back: WorldSave = toml::from_str(&text).expect("the file again");
    assert_eq!(Taken::from_text(&back.taken), taken);
    let old: WorldSave = toml::from_str("name = \"Old\"\nseed = 7\n").expect("an old file");
    assert!(old.taken.is_empty());
}

/// Only rocks are left out for a taken id: a tree is never a rock.
#[test]
fn spare_leaves_out_the_taken_rocks_only() {
    let mut props = vec![prop(Plant::Rock, 1.0, 1.0), prop(Plant::Oak, 1.0, 1.0)];
    let mut taken = Taken::default();
    taken.add(id_of(9, &props[0]));
    taken.add(id_of(9, &props[1]));
    spare(&mut props, 9, &taken);
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].plant, Plant::Oak);
}

/// Taking a rock removes it and its prop from the stand, once, and the stand says where it stood.
#[test]
fn a_taken_rock_leaves_the_stand() {
    let mut stand = stand();
    assert_eq!(stand.rocks().len(), 2);
    assert_eq!(stand.props().len(), 3);
    let first = stand.rocks()[0];
    assert!(!stand.dirty());
    assert!(stand.remove(first.id));
    assert!(!stand.remove(first.id));
    assert!(stand.dirty());
    assert_eq!(stand.rocks().len(), 1);
    assert_eq!(stand.props().len(), 2);
    assert!(stand.rocks().iter().all(|r| r.id != first.id));
    assert_eq!(stand.settle(), vec![first.at]);
    assert!(!stand.dirty());
}

/// A point on the body of a rock finds that rock, and a point in the air finds none.
#[test]
fn a_point_on_a_rock_finds_it() {
    let stand = stand();
    let rock = stand.rocks()[0];
    let on_it = rock.mid + Vec3::new(0.3, 0.2, 0.0);
    assert_eq!(stand.near(on_it).map(|r| r.id), Some(rock.id));
    assert_eq!(stand.near(rock.mid + Vec3::Y * 20.0), None);
    assert_eq!(stand.near(Vec3::new(5.0, 5.0, 5.0)), None);
}

/// A rock weighs its volume times the density of its stone.
#[test]
fn a_rock_weighs_its_volume_times_its_density() {
    let rock = stand().rocks()[0];
    let volume = 2.0 / 3.0 * std::f32::consts::PI * 0.7 * 0.7 * 0.9;
    assert!((rock.mass(3000.0) - volume * 3000.0).abs() < 1.0);
    assert!(rock.mass(917.0) < rock.mass(3000.0));
}
