//! Letting players in: the status anyone may ask for, the checks of a hello, and the welcome.

use std::fmt::Write;

use super::Room;
use super::seat::{Seat, Sent};
use crate::host::{ConnId, Outbox};
use crate::log::PlayerId;
use crate::wire::{
    Hello, Kind, PROTOCOL, PlayerInfo, Reason, ServerMsg, Status, Welcome, clean_name,
};

impl Room {
    /// Tells `conn` what the server is. It is answered before any check.
    pub(super) fn status(&self, conn: ConnId, out: &mut Outbox) {
        let status = Status {
            name: self.config.name.clone(),
            players: u16::try_from(self.seats.iter().flatten().count()).unwrap_or(u16::MAX),
            cap: self.config.cap,
            version: self.config.version.clone(),
            protocol: PROTOCOL,
        };
        out.send(conn, ServerMsg::Status(status));
    }

    /// Lets `conn` in with a welcome and tells the others, or refuses it and closes it. A
    /// hello on a connection that already joined is ignored. A player let in who already holds
    /// a seat from an older connection takes it over: a key never holds two seats. The record
    /// kept of the player is checked before it is handed back.
    pub(super) fn hello(&mut self, conn: ConnId, hello: Hello, now: f64, out: &mut Outbox) {
        if self.conns.get(&conn).is_none_or(|open| open.seat.is_some()) {
            return;
        }
        let refusal = self.refusal(&hello);
        let free = self.seat_for(&hello.key, refusal.is_none(), out);
        let Some(slot) = free.filter(|_| refusal.is_none()) else {
            let reason = refusal.unwrap_or(Reason::Full);
            self.conns.remove(&conn);
            out.send(conn, ServerMsg::Refused(reason));
            out.close(conn, reason);
            return;
        };
        self.mend(&hello.key);
        let id = self.id_for(&hello.key);
        let name = Some(clean_name(&hello.name)).filter(|name| !name.is_empty());
        let info = PlayerInfo {
            id,
            name: name.unwrap_or_else(|| format!("Robot {}", id.0)),
        };
        out.send(conn, ServerMsg::Welcome(self.welcome(id, &hello.key, now)));
        self.tell_hunt(conn, out);
        self.broadcast(&ServerMsg::Joined(info.clone()), out);
        self.seat(slot, Seat::new(conn, hello.key, info));
        if let Some(open) = self.conns.get_mut(&conn) {
            open.seat = Some(slot);
        }
    }

    /// Returns why `hello` cannot join, checking the protocol and then the password.
    fn refusal(&self, hello: &Hello) -> Option<Reason> {
        if hello.protocol < PROTOCOL {
            return Some(Reason::Outdated);
        }
        if hello.protocol > PROTOCOL {
            return Some(Reason::Newer);
        }
        let given = hello.password.as_deref().unwrap_or_default();
        let wrong = self
            .config
            .password
            .as_deref()
            .is_some_and(|password| !same(password, given));
        wrong.then_some(Reason::WrongPassword)
    }

    /// Returns the id of the player with `key`: the one they had this session if nobody holds
    /// it now, else a new one. An older seat of the key is freed before, so the id is kept.
    fn id_for(&mut self, key: &[u8; 32]) -> PlayerId {
        let held = |id: PlayerId| self.seats.iter().flatten().any(|seat| seat.info.id == id);
        if let Some(id) = self.ids.get(key).copied().filter(|id| !held(*id)) {
            return id;
        }
        let id = PlayerId(self.next_id);
        self.next_id += 1;
        self.ids.insert(*key, id);
        id
    }

    /// Returns what the player `you` with `key` needs to enter the world at `now`.
    fn welcome(&self, you: PlayerId, key: &[u8; 32], now: f64) -> Welcome {
        Welcome {
            you,
            seed: self.world.seed,
            clock: self.clock(now),
            changes: self.world.changes.clone(),
            atlas: self.world.atlas.clone(),
            record: self
                .world
                .players
                .get(&hex(key))
                .map(|member| member.record()),
            players: self.players(),
            explored: self.explored_list(),
            waypoints: self.waypoints(),
        }
    }

    /// Puts `seat` in `slot`, forgetting what anyone was sent of the slot's last player.
    fn seat(&mut self, slot: usize, seat: Seat) {
        self.seats[slot] = Some(seat);
        self.sent[slot].fill([Sent::NONE; Kind::COUNT]);
        for row in &mut self.sent {
            row[slot] = [Sent::NONE; Kind::COUNT];
        }
    }
}

/// Returns true when both passwords are equal, taking the same time wherever they differ.
fn same(expected: &str, given: &str) -> bool {
    let diff = expected
        .bytes()
        .zip(given.bytes())
        .fold(0u8, |diff, (a, b)| diff | (a ^ b));
    std::hint::black_box(diff) == 0 && expected.len() == given.len()
}

/// Returns `key` in lowercase hexadecimal, as the world file stores it.
pub(super) fn hex(key: &[u8; 32]) -> String {
    key.iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}
