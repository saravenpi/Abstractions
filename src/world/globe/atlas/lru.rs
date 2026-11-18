//! A shared memory of the last 256 cells used, safe to read from every thread that builds chunks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use super::Cell;

/// Cells kept before the one used longest ago is forgotten.
const CAPACITY: usize = 256;

/// The cells, each with the tick it was last used at, and the clock of ticks.
type Slots<V> = (HashMap<Cell, (Arc<V>, u64)>, u64);

/// A memory of values by cell that forgets the least recently used past [`CAPACITY`].
pub struct Lru<V> {
    /// The values and the clock, behind one lock.
    slots: Mutex<Slots<V>>,
}

impl<V> Default for Lru<V> {
    fn default() -> Self {
        Self {
            slots: Mutex::new((HashMap::new(), 0)),
        }
    }
}

impl<V> Lru<V> {
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
        if slots.len() >= CAPACITY {
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
