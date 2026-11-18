//! A shared memory of the cells used last, safe to read from every thread that builds chunks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use super::Cell;

/// Fewest cells a memory keeps: more than the 216 of the home planet, which are never forgotten.
const LEAST: usize = 256;

/// Most cells a memory keeps. A flow map near a cube corner reads the grids of about 250 cells, and
/// several threads build maps at once, so a large planet keeps eight times that.
pub const MOST: usize = 2048;

/// Returns the cells a memory keeps on a planet of `cells` cells per face side: all of them, held between
/// [`LEAST`] and [`MOST`].
pub fn kept(cells: i32) -> usize {
    let all = 6 * cells.max(1) as usize * cells.max(1) as usize;
    all.clamp(LEAST, MOST)
}

/// The cells, each with the tick it was last used at, and the clock of ticks.
type Slots<V> = (HashMap<Cell, (Arc<V>, u64)>, u64);

/// A memory of values by cell that forgets the least recently used past its capacity.
pub struct Lru<V> {
    /// The values and the clock, behind one lock.
    slots: Mutex<Slots<V>>,
    /// Cells kept before the one used longest ago is forgotten.
    capacity: usize,
}

impl<V> Lru<V> {
    /// Returns an empty memory that keeps `capacity` cells.
    pub fn sized(capacity: usize) -> Self {
        Self {
            slots: Mutex::new((HashMap::new(), 0)),
            capacity,
        }
    }

    /// Returns the value of `cell`, building it with `build` when it is not kept.
    ///
    /// The lock is never held while building, so a build may read other cells; two threads may build the
    /// same cell at once, and both get the same value since every build is a pure function of the cell.
    pub fn get(&self, cell: Cell, build: impl FnOnce() -> V) -> Arc<V> {
        if let Some(found) = self.touch(cell) {
            return found;
        }
        let made = Arc::new(build());
        let mut guard = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        let (slots, clock) = &mut *guard;
        *clock += 1;
        if slots.len() >= self.capacity {
            let oldest = slots.iter().min_by_key(|(_, (_, t))| *t).map(|(c, _)| *c);
            oldest.map(|c| slots.remove(&c));
        }
        let entry = slots.entry(cell).or_insert((made, *clock));
        entry.0.clone()
    }

    /// Returns the kept value of `cell`, marking it as just used.
    fn touch(&self, cell: Cell) -> Option<Arc<V>> {
        let mut guard = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        let (slots, clock) = &mut *guard;
        *clock += 1;
        let tick = *clock;
        slots.get_mut(&cell).map(|(value, t)| {
            *t = tick;
            value.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::globe::Face;

    /// A memory keeps every cell of the home planet and of smaller ones, 2,048 cells of a 40 km planet,
    /// and forgets the cell used longest ago past that.
    #[test]
    fn memories_are_sized_by_the_planet() {
        assert_eq!([kept(1), kept(6), kept(8), kept(40)], [256, 256, 384, 2048]);
        let memory = Lru::sized(kept(8));
        let cell = |i: i32| Cell {
            face: Face::PosX,
            i,
            j: 0,
        };
        let mut built = 0;
        let mut read = |i: i32| *memory.get(cell(i), || built += 1);
        (0..384).for_each(&mut read);
        (0..384).for_each(&mut read);
        read(384);
        read(1);
        read(0);
        assert_eq!(built, 386);
    }
}
