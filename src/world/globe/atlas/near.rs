//! The cells each thread read: the hot path reads a cell it already holds without a lock or a count.
//!
//! The last cell read is checked first, since a chunk reads one cell for hundreds of samples in a row;
//! the others are found by hash. A thread forgets all of them past [`HELD`].

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use super::mix::BuildMix;
use super::{Cell, Kind};

/// Cells a thread holds before it forgets them all.
const HELD: usize = 2048;

/// What tells one held cell from another: the planet fingerprint, the kind of value and the cell.
type Key = (u64, Kind, Cell);

/// A held value.
type Value = Arc<dyn Any + Send + Sync>;

/// The cells of one thread: the last one read, and all of them by key.
#[derive(Default)]
struct Hold {
    /// The cell read last, per kind of value.
    last: [Option<(Key, Value)>; 3],
    /// Every held cell.
    all: HashMap<Key, Value, BuildMix>,
}

thread_local! {
    static HOLD: RefCell<Hold> = RefCell::new(Hold::default());
}

/// Returns the held value of `key`, making it the last one read of its kind.
fn find(hold: &mut Hold, key: Key) -> Option<&Value> {
    let slot = key.1 as usize;
    let last = hold.last[slot].as_ref().is_some_and(|(k, _)| *k == key);
    if !last {
        let found = hold.all.get(&key)?.clone();
        hold.last[slot] = Some((key, found));
    }
    hold.last[slot].as_ref().map(|(_, v)| v)
}

/// Returns what `read` makes of the value of `key`, taken from this thread's held cells or else from
/// `fetch`, which may itself read other cells. A held value is read in place, without a count, so
/// `read` must not read another cell.
pub fn near<V: Any + Send + Sync, R>(
    key: Key,
    fetch: impl FnOnce() -> Arc<V>,
    read: impl FnOnce(&V) -> R,
) -> R {
    let mut read = Some(read);
    let held = HOLD.with(|hold| {
        let mut hold = hold.borrow_mut();
        let value = find(&mut hold, key)?.downcast_ref::<V>()?;
        read.take().map(|read| read(value))
    });
    if let Some(done) = held {
        return done;
    }
    let value = fetch();
    HOLD.with(|hold| {
        let mut hold = hold.borrow_mut();
        if hold.all.len() >= HELD {
            hold.all.clear();
        }
        hold.all.insert(key, value.clone());
        hold.last[key.1 as usize] = Some((key, value.clone()));
    });
    read.take()
        .map(|read| read(&value))
        .expect("a value is read once")
}
