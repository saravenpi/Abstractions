//! The path a drop of water takes down the drainage grid.

use super::Drainage;

impl Drainage {
    /// Returns the path of the water from `cell` down to its end, both included: a sea cell, or the bottom of
    /// a closed basin.
    pub fn course(&self, cell: usize) -> Vec<usize> {
        let mut path = vec![cell];
        while let Some(to) = self.next(path[path.len() - 1]) {
            path.push(to);
        }
        path
    }
}
