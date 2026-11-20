//! Spears that stand and items that are held: what the room keeps of them and who it tells.

use crate::host::{ConnId, Outbox};
use crate::log::PlayerId;
use crate::room::Room;
use crate::wire::{Event, HuntWorld, ITEM_MAX, ServerMsg, Spear, SpearKey};

/// Seconds between two sweeps of the hunt for what lasted its time.
const SWEEP_EVERY: f64 = 1.0;

impl Room {
    /// Plants the spear `spear` of `by` and tells the others. A spear with an item that is too
    /// long, or a place or direction that is not a number, is dropped.
    pub(super) fn stuck(&mut self, conn: ConnId, by: PlayerId, spear: Spear, out: &mut Outbox) {
        let finite = spear.at.iter().chain(&spear.dir).all(|x| x.is_finite());
        if !finite || spear.item.len() > ITEM_MAX {
            return;
        }
        let key = SpearKey {
            owner: by,
            ..spear.key
        };
        let spear = Spear { key, ..spear };
        self.hunting.plant(spear.clone());
        self.dirty = true;
        self.send_others(conn, &ServerMsg::Hunt(Event::Stuck(spear)), out);
    }

    /// Takes the spear `key` out for `by` and tells the others, when it stood.
    pub(super) fn pulled(&mut self, conn: ConnId, by: PlayerId, key: SpearKey, out: &mut Outbox) {
        if self.hunting.pull(key) {
            self.dirty = true;
            self.send_others(conn, &ServerMsg::Hunt(Event::Pulled { key, by }), out);
        }
    }

    /// Keeps what the player on `conn` holds and tells the others. An item that is too long
    /// is dropped.
    pub(super) fn held(
        &mut self,
        conn: ConnId,
        by: PlayerId,
        item: Option<Vec<u8>>,
        out: &mut Outbox,
    ) {
        if item.as_ref().is_some_and(|item| item.len() > ITEM_MAX) {
            return;
        }
        let seat = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_mut());
        if let Some(seat) = seat {
            seat.held.clone_from(&item);
        }
        self.send_others(conn, &ServerMsg::Hunt(Event::Held { by, item }), out);
    }

    /// Tells the player on `conn` the state of the hunt, when there is anything to tell.
    pub(in crate::room) fn tell_hunt(&self, conn: ConnId, out: &mut Outbox) {
        let held = self.seats.iter().flatten();
        let held = held.filter_map(|seat| Some((seat.info.id, seat.held.clone()?)));
        let world = self.hunting.world(held.collect());
        if world != HuntWorld::default() {
            out.send(conn, ServerMsg::Hunt(Event::World(world)));
        }
    }

    /// Forgets what lasted its time, at most once every [`SWEEP_EVERY`] seconds.
    pub(in crate::room) fn sweep(&mut self, now: f64) {
        if now - self.swept < SWEEP_EVERY {
            return;
        }
        self.swept = now;
        if self.hunting.prune(self.clock(now)) {
            self.dirty = true;
        }
    }
}
