//! A buffer whose body changes frame: its snapshots are carried over, or forgotten.

use super::{Buffer, Snap};

impl<E: Copy + Default> Buffer<E> {
    /// Replaces each snapshot, oldest first, with what `f` returns for it, and drops the ones
    /// it returns nothing for. Call it when the frame of the body changes, with `f` converting a
    /// snapshot to the new frame. The delays and the jitter measures stay. A snapshot that `f`
    /// moves to a time no later than the one kept before it is dropped, so the order holds.
    pub fn rebase(&mut self, f: impl FnMut(Snap<E>) -> Option<Snap<E>>) {
        self.ring.rebase(f);
    }

    /// Forgets every snapshot but keeps the delays and the jitter measures. Call it when the
    /// frame of the body changes and the snapshots cannot be converted, so no position is
    /// blended across two frames.
    pub fn reset(&mut self) {
        self.ring.clear();
        self.spread.forget();
    }
}
