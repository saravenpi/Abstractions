//! Tests of the explored ground that comes from elsewhere: bad data from a server keeps
//! nothing, and the runs of a world file come back whole.

use abstractions_net::leaves::{Leaves, Run, key};
use abstractions_net::room::BODIES_MAX;

use crate::map::explored::Explored;
use crate::saves::WorldSave;
use crate::universe::{BodyId, Universe};

/// A server cannot make the game keep ground for empty lists or keys that name no leaf.
#[test]
fn bad_ground_from_a_server_keeps_nothing() {
    let home = Universe::new(3).home();
    let mut explored = Explored::default();
    explored.merge_runs(home, &[]);
    explored.merge_runs(
        home,
        &[Run {
            gap: key(6, 4, 0),
            len: 3,
        }],
    );
    explored.merge_runs(
        home,
        &[Run {
            gap: u64::MAX,
            len: u64::MAX,
        }],
    );
    explored.merge_runs(
        home,
        &[Run {
            gap: key(1, 4, 250),
            len: 20,
        }],
    );
    assert_eq!(explored.bodies().count(), 0);
}

/// A server cannot make the game keep ground for more bodies than the room keeps.
#[test]
fn a_server_cannot_make_the_game_keep_every_body() {
    let home = Universe::new(3).home();
    let mut explored = Explored::default();
    let good = [Run {
        gap: key(1, 8, 3),
        len: 2,
    }];
    for index in 0..=u8::MAX {
        explored.merge_runs(BodyId { index, ..home }, &good);
    }
    assert_eq!(explored.bodies().count(), BODIES_MAX);
}

/// Explored runs survive a trip through the world file, a high key included.
#[test]
fn saved_runs_survive_the_file() {
    let body = Universe::new(3).home();
    let mut leaves = Leaves::default();
    for morton in [0, 1, 2, 90_000] {
        leaves.insert(key(5, 12, morton));
    }
    let mut save = toml::from_str::<WorldSave>("[[explored]]\nruns = []").expect("reads");
    save.explored[0].body = body;
    save.explored[0].runs = leaves.runs();
    let text = toml::to_string(&save).expect("writes");
    let back: WorldSave = toml::from_str(&text).expect("reads back");
    assert_eq!(back.explored, save.explored);
    let loaded = Explored::of(&back);
    assert_eq!(loaded.leaves(body).expect("loaded"), &leaves);
}
