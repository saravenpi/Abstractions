//! What the room keeps of each player between sessions: the record a player sends, checked
//! when it comes in and again before it is handed back.

use super::Room;
use super::join::hex;
use crate::host::ConnId;
use crate::store::place;
use crate::wire::PlayerRecord;

impl Room {
    /// Keeps the place, ship and saved places of the player on `conn`, under their key. Only
    /// the connection that holds the seat of the key is read, and the stream delivers its
    /// records in order, so the record kept is always the newest of the newest session. A
    /// place that is not one is repaired at once.
    pub(super) fn record(&mut self, conn: ConnId, record: PlayerRecord) {
        let Some(seat) = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_ref())
        else {
            return;
        };
        let key = seat.key;
        let member = self.world.players.entry(hex(&key)).or_default();
        member.name.clone_from(&seat.info.name);
        member.place = record.place;
        if !record.ship.is_empty() {
            member.ship = record.ship;
        }
        member.marks = record.marks;
        if !record.pack.is_empty() {
            member.pack = record.pack;
        }
        self.dirty = true;
        self.mend(&key);
    }

    /// Repairs the place kept of the player with `key` when it is not a place a body can be
    /// at, and says so on the standard output, which is the log of a server.
    pub(super) fn mend(&mut self, key: &[u8; 32]) {
        let Some(member) = self.world.players.get_mut(&hex(key)) else {
            return;
        };
        if let Some(line) = place::mend(member) {
            println!("{line}");
            self.dirty = true;
        }
    }
}
