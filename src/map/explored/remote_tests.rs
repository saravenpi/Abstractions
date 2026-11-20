//! Tests of the explored ground that comes from elsewhere: bad data from a server keeps
//! nothing, and the words of a world file come back whole.

use abstractions_net::room::BODIES_MAX;

use crate::map::explored::Explored;
use crate::map::explored::grid::{CELLS, WORDS};
use crate::saves::WorldSave;
use crate::universe::{BodyId, Universe};

/// A server cannot make the game keep ground for empty lists, lists of the wrong size, cells
/// off the grid, or more bodies than the room keeps.
#[test]
fn bad_ground_from_a_server_keeps_nothing() {
    let universe = Universe::new(3);
    let home = universe.home();
    let mut explored = Explored::default();
    explored.reveal_cells(home, &[]);
    explored.reveal_cells(home, &[CELLS as u32, u32::MAX]);
    explored.merge_words(home, &[]);
    explored.merge_words(home, &[u64::MAX; 3]);
    explored.merge_words(home, &vec![0; WORDS]);
    assert_eq!(explored.bodies().count(), 0);
    for index in 0..=u8::MAX {
        let body = BodyId { index, ..home };
        explored.reveal_cells(body, &[1]);
        explored.merge_words(body, &vec![u64::MAX; WORDS]);
    }
    assert_eq!(explored.bodies().count(), BODIES_MAX);
}

/// Explored words survive a trip through the world file, the top bit of a word included.
#[test]
fn saved_words_survive_the_file() {
    let body = Universe::new(3).home();
    let mut words = vec![0; WORDS];
    words[0] = (1 << 63) | 5;
    words[WORDS - 1] = u64::MAX;
    let mut save = toml::from_str::<WorldSave>("[[explored]]\nwords = [0]").expect("reads");
    save.explored[0].body = body;
    save.explored[0].words.clone_from(&words);
    let text = toml::to_string(&save).expect("writes");
    let back: WorldSave = toml::from_str(&text).expect("reads back");
    assert_eq!(back.explored, save.explored);
    assert_eq!(Explored::of(&back).words(body).expect("loaded"), &words[..]);
}
