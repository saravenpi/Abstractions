//! Tests of the drainage grid's geometry.

use super::grid::Grid;

/// The neighbours of a cell inside a face, taken from the index, are the ones its directions give.
#[test]
fn neighbours_inside_a_face_follow_their_directions() {
    let grid = Grid { n: 16 };
    for index in (0..grid.len()).step_by(3) {
        let (face, i, j) = grid.split(index);
        assert_eq!(grid.around(index), grid.across(face, i, j), "cell {index}");
    }
}
