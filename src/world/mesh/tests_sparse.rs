//! Tests of the cells of coarse nodes: fine near the eye, half and a quarter as many per side for
//! the far nodes the air has faded.

use super::GRID_POINTS;
use super::node::node_cells;
use super::tune::SPARSE_LEVELS;

/// The leaves get 8 cells per side, the near coarse nodes 16 and 32, and the far ones fewer again,
/// never more than a nearer level and never fewer than 8.
#[test]
fn far_nodes_get_fewer_cells() {
    let full = GRID_POINTS - 1;
    assert_eq!(node_cells(0), full / 4);
    assert_eq!(node_cells(1), full / 2);
    assert_eq!(node_cells(SPARSE_LEVELS.0 - 1), full);
    assert_eq!(node_cells(SPARSE_LEVELS.0), full / 2);
    assert_eq!(node_cells(SPARSE_LEVELS.1), full / 4);
    assert_eq!(node_cells(20), full / 4);
    let cells: Vec<usize> = (2..=20).map(node_cells).collect();
    assert!(cells.windows(2).all(|pair| pair[1] <= pair[0]));
    assert!(cells.iter().all(|&n| n >= 8));
}
