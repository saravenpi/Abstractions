//! Tests of the explored ground: the grid, the reach, the network lists and the save.

use bevy::math::DVec3;

use super::grid::{CELLS, COLS, WORDS, cell_of, centre, disc};
use super::{Explored, REACH_CAP, REACH_FOOT, reach};
use crate::saves::{Place, WorldSave};
use crate::universe::{EARTH_RADIUS, Universe};

/// Every cell holds its own centre.
#[test]
fn every_cell_holds_its_centre() {
    for cell in 0..CELLS as u32 {
        assert_eq!(cell_of(centre(cell)), cell);
    }
}

/// Longitude 0 is +X, east is -Z, and row 0 is at the south pole.
#[test]
fn the_grid_starts_at_the_south_and_runs_east() {
    assert_eq!(cell_of(DVec3::new(1.0, -1e4, -0.001).normalize()), 0);
    assert_eq!(
        cell_of(DVec3::new(0.0, 0.0, -1.0)) % COLS as u32,
        COLS as u32 / 4
    );
}

/// Returns the cells of a disc of `metres` on the home planet around `dir`, without repeats.
fn cells(dir: DVec3, metres: f64) -> Vec<u32> {
    let mut cells = Vec::new();
    disc(dir, metres / EARTH_RADIUS, |cell| cells.push(cell));
    cells.sort_unstable();
    cells.dedup();
    cells
}

/// A walk uncovers about the cells of a 300 m disc, and none farther than the disc and a cell.
#[test]
fn a_walk_uncovers_the_cells_around_it() {
    let dir = DVec3::new(1.0, 0.3, -0.4).normalize();
    let found = cells(dir, REACH_FOOT);
    let side = std::f64::consts::TAU * EARTH_RADIUS / COLS as f64;
    let expected = std::f64::consts::PI * REACH_FOOT * REACH_FOOT / (side * side);
    println!("{} cells for {expected:.0} expected", found.len());
    assert!((found.len() as f64 - expected).abs() < expected * 0.4);
    let far = (REACH_FOOT + side) / EARTH_RADIUS;
    assert!(
        found
            .iter()
            .all(|&cell| centre(cell).angle_between(dir) < far)
    );
}

/// A disc over a pole wraps round every longitude.
#[test]
fn a_disc_over_the_pole_takes_every_column() {
    let found = cells(DVec3::new(0.001, 1.0, 0.0).normalize(), REACH_CAP);
    let top = (CELLS - COLS) as u32;
    assert!((top..CELLS as u32).all(|cell| found.contains(&cell)));
}

/// The reach is 300 m on the ground, the horizon higher up, and never past 2 km.
#[test]
fn the_reach_grows_with_height_to_a_cap() {
    assert_eq!(reach(-5.0, EARTH_RADIUS), REACH_FOOT);
    assert!((reach(100.0, EARTH_RADIUS) - 1100.0).abs() < 1.0);
    assert_eq!(reach(5000.0, EARTH_RADIUS), REACH_CAP);
}

/// Cells uncovered here are handed to the network once; cells from the server never are.
#[test]
fn only_local_cells_are_sent_and_only_once() {
    let body = Universe::new(3).home();
    let mut explored = Explored::default();
    explored.reveal_cells(body, &[5, 6]);
    assert!(explored.take_new(body).is_empty());
    let before = explored.version(body);
    explored.reveal(body, centre(5), 1e-6);
    assert_eq!(explored.version(body), before);
    explored.reveal(body, centre(9), 1e-6);
    assert_eq!(explored.take_new(body), vec![9]);
    assert!(explored.take_new(body).is_empty());
    let mut words = vec![0; WORDS];
    words[0] = 1 << 40;
    explored.merge_words(body, &words);
    assert_eq!(
        explored.words(body).unwrap()[0],
        (1 << 40) | (1 << 9) | 0b110_0000
    );
}

/// A saved bitset loads as it was; a file without one is uncovered around its last place.
#[test]
fn saves_load_their_ground() {
    let universe = Universe::new(3);
    let body = universe.home();
    let mut explored = Explored::default();
    explored.reveal_cells(body, &[1, 700]);
    let mut save = WorldSave {
        seed: 3,
        place: Some(Place {
            body,
            dir: [0.0, 0.0, 1.0],
            altitude: 2.0,
        }),
        ..WorldSave::default()
    };
    let words = explored.words(body).unwrap().to_vec();
    let old = Explored::of(&save);
    assert!(old.words(body).unwrap()[cell_of(DVec3::Z) as usize / 64] != 0);
    save.explored = toml::from_str::<WorldSave>(&format!("[[explored]]\nwords = {words:?}"))
        .map(|file| file.explored)
        .unwrap();
    save.explored[0].body = body;
    assert_eq!(Explored::of(&save).words(body).unwrap(), &words[..]);
}
