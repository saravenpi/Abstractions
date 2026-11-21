//! The drainages built so far in this process, by planet fingerprint: a planet entered again, or built
//! again by a test, takes its grid from here instead of reading 393,216 heights once more.

use std::sync::{Arc, Mutex, OnceLock};

use super::Drainage;

/// Planets whose drainage is kept: the last ones built.
const KEPT: usize = 8;

/// The slot of one planet: empty until its drainage is built, filled once.
type Slot = Arc<OnceLock<Arc<Drainage>>>;

/// The slots of the planets built so far, the oldest first.
static BUILT: Mutex<Vec<(u64, Slot)>> = Mutex::new(Vec::new());

/// Returns the slot of the planet of fingerprint `print`, made empty when it has none, dropping the oldest
/// planet past [`KEPT`].
fn slot(print: u64) -> Slot {
    let mut built = BUILT.lock().unwrap_or_else(|held| held.into_inner());
    if let Some((_, slot)) = built.iter().find(|(known, _)| *known == print) {
        return slot.clone();
    }
    if built.len() >= KEPT {
        built.remove(0);
    }
    let fresh = Slot::default();
    built.push((print, fresh.clone()));
    fresh
}

/// Returns the drainage of the planet of fingerprint `print`, from `build` when no one built it yet. A
/// second caller for the same planet waits for the first instead of building it again.
pub fn of(print: u64, build: impl FnOnce() -> Drainage) -> Arc<Drainage> {
    slot(print).get_or_init(|| Arc::new(build())).clone()
}
