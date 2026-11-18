//! What joined players do on the reliable stream: keep their record, change the world, leave.

use super::Room;
use super::conn::CHANGES_PER_SECOND;
use super::join::hex;
use crate::host::{ConnId, Outbox};
use crate::log::{Change, Entry};
use crate::wire::{PlayerRecord, ServerMsg};

impl Room {
    /// Returns the seat of `conn`, when it joined.
    pub(super) fn seat_of(&self, conn: ConnId) -> Option<usize> {
        self.conns.get(&conn).and_then(|open| open.seat)
    }

    /// Keeps the place and ship of the player on `conn`, under their key.
    pub(super) fn record(&mut self, conn: ConnId, record: PlayerRecord) {
        let Some(seat) = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_ref())
        else {
            return;
        };
        let member = self.world.players.entry(hex(&seat.key)).or_default();
        member.name.clone_from(&seat.info.name);
        member.place = record.place;
        member.ship = record.ship;
    }

    /// Adds the change of the player on `conn` to the log and tells everyone, the player
    /// included. A body already visited, or one change too many this second, is dropped.
    pub(super) fn change(&mut self, conn: ConnId, change: Change, now: f64, out: &mut Outbox) {
        let seat = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_ref());
        let Some(player) = seat.map(|seat| seat.info.id) else {
            return;
        };
        let log = &self.world.changes.entries;
        let known = log.iter().any(|entry| entry.change == change);
        let open = self.conns.get_mut(&conn);
        if known || !open.is_some_and(|open| open.changes.allow(now, CHANGES_PER_SECOND)) {
            return;
        }
        let time = self.clock(now);
        self.world.changes.push(player, time, change.clone());
        let entry = Entry {
            player,
            time,
            change,
        };
        self.broadcast(&ServerMsg::Change(entry), out);
    }

    /// Frees the seat of `conn` and tells the others the player left. The player's saved
    /// record stays.
    pub(super) fn leave(&mut self, conn: ConnId, out: &mut Outbox) {
        let Some(slot) = self.conns.get_mut(&conn).and_then(|open| open.seat.take()) else {
            return;
        };
        if let Some(seat) = self.seats[slot].take() {
            self.broadcast(&ServerMsg::Left(seat.info.id), out);
        }
    }

    /// Lets the player on `conn` leave and forgets the connection.
    pub(super) fn forget(&mut self, conn: ConnId, out: &mut Outbox) {
        self.leave(conn, out);
        self.conns.remove(&conn);
    }

    /// Sends `msg` to every joined player.
    pub(super) fn broadcast(&self, msg: &ServerMsg, out: &mut Outbox) {
        for seat in self.seats.iter().flatten() {
            out.send(seat.conn, msg.clone());
        }
    }
}
