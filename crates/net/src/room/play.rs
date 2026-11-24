//! What joined players do on the reliable stream: change the world and leave, and the one seat
//! a player key may hold.

use super::Room;
use super::conn::CHANGES_PER_SECOND;
use crate::host::{ConnId, Outbox};
use crate::log::{Change, Entry};
use crate::wire::{Reason, ServerMsg};

impl Room {
    /// Returns the seat of `conn`, when it joined.
    pub(super) fn seat_of(&self, conn: ConnId) -> Option<usize> {
        self.conns.get(&conn).and_then(|open| open.seat)
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
        self.dirty = true;
        let entry = Entry {
            player,
            time,
            change,
        };
        self.broadcast(&ServerMsg::Change(entry), out);
    }

    /// Frees the seat of `conn` and tells the others the player left, clearing their waypoint
    /// first when they had one. The player's saved record stays.
    pub(super) fn leave(&mut self, conn: ConnId, out: &mut Outbox) {
        let Some(slot) = self.conns.get_mut(&conn).and_then(|open| open.seat.take()) else {
            return;
        };
        let Some(seat) = self.seats[slot].take() else {
            return;
        };
        if seat.waypoint.is_some() {
            let player = seat.info.id;
            let waypoint = None;
            self.broadcast(&ServerMsg::Waypoint { player, waypoint }, out);
        }
        self.leave_ships(seat.info.id, out);
        self.broadcast(&ServerMsg::Left(seat.info.id), out);
    }

    /// Returns a free seat for the player with `key`, or nothing when the room is full. When
    /// `admit` is true the player is let in, so an older connection that holds a seat under the
    /// same key is sent away first and its seat freed: the newest join wins, and nothing the
    /// older connection sends afterwards is read.
    pub(super) fn seat_for(
        &mut self,
        key: &[u8; 32],
        admit: bool,
        out: &mut Outbox,
    ) -> Option<usize> {
        let mut held = self.seats.iter().flatten();
        let older = held.find(|seat| seat.key == *key).map(|seat| seat.conn);
        if let Some(old) = older.filter(|_| admit) {
            out.send(old, ServerMsg::Kick(Reason::Replaced));
            self.forget(old, out);
            out.close(old, Reason::Replaced);
        }
        self.seats.iter().position(Option::is_none)
    }

    /// Lets the player on `conn` leave and forgets the connection.
    pub(super) fn forget(&mut self, conn: ConnId, out: &mut Outbox) {
        self.leave(conn, out);
        self.conns.remove(&conn);
    }

    /// Sends `msg` to every joined player but the one on `conn`.
    pub(super) fn send_others(&self, conn: ConnId, msg: &ServerMsg, out: &mut Outbox) {
        for seat in self.seats.iter().flatten().filter(|seat| seat.conn != conn) {
            out.send(seat.conn, msg.clone());
        }
    }

    /// Sends `msg` to every joined player.
    pub(super) fn broadcast(&self, msg: &ServerMsg, out: &mut Outbox) {
        for seat in self.seats.iter().flatten() {
            out.send(seat.conn, msg.clone());
        }
    }
}
