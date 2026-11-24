//! Shared ship ownership and authority inside the room: who flies which ship and who sends its state.

use super::Room;
use super::seat::{Recent, Sent};
use crate::host::{ConnId, Outbox};
use crate::log::PlayerId;
use crate::wire::{Record, ServerMsg, ShipAct, ShipEvent};

/// State buffer and control ownership for one player's ship in the room.
#[derive(Debug)]
pub(super) struct ShipHold {
    /// Player id that owns the ship.
    pub(super) id: PlayerId,
    /// Player currently authoritative over state datagrams, or `None` when parked unattended.
    pub(super) authority: Option<PlayerId>,
    /// Player currently sitting in the pilot seat.
    pub(super) pilot: Option<PlayerId>,
    /// True after the owner left the session while a friend still pilots this ship.
    pub(super) owner_gone: bool,
    /// Circular buffer of recent ship state records.
    pub(super) recent: Recent,
}

/// Pre-allocated table of ship holds and per-recipient send watermarks (zero allocations in `tick`).
#[derive(Debug)]
pub(super) struct Fleet {
    /// Holds indexed by pre-allocated slot.
    pub(super) holds: Vec<Option<ShipHold>>,
    /// Per recipient seat index, the last watermark sent for each hold slot.
    pub(super) sent: Vec<Vec<Sent>>,
}

impl Fleet {
    /// Allocates capacity for `cap` active recipients and `2 * cap` active or borrowed ships.
    pub(super) fn new(cap: usize) -> Self {
        let slots = cap.saturating_mul(2);
        Self {
            holds: (0..slots).map(|_| None).collect(),
            sent: vec![vec![Sent::NONE; slots]; cap],
        }
    }

    /// Returns the hold slot index for `ship_id` when present.
    pub(super) fn slot_of(&self, ship_id: PlayerId) -> Option<usize> {
        self.holds
            .iter()
            .position(|hold| hold.as_ref().is_some_and(|h| h.id == ship_id))
    }

    /// Finds the hold slot of `ship_id` or allocates a free slot owned and authored by `ship_id`.
    fn ensure(&mut self, ship_id: PlayerId) -> Option<usize> {
        if let Some(idx) = self.slot_of(ship_id) {
            return Some(idx);
        }
        let idx = self.holds.iter().position(Option::is_none)?;
        self.holds[idx] = Some(ShipHold {
            id: ship_id,
            authority: Some(ship_id),
            pilot: None,
            owner_gone: false,
            recent: Recent::default(),
        });
        for row in &mut self.sent {
            row[idx] = Sent::NONE;
        }
        Some(idx)
    }
}

impl Room {
    /// Keeps an incoming `Kind::Ship` record when `player` currently holds authority over that ship.
    pub(super) fn ingest_ship(&mut self, slot: usize, player: PlayerId, record: &Record<'_>) {
        let ship_id = record.head.player;
        let idx = match self.fleet.slot_of(ship_id) {
            Some(idx) => idx,
            None if ship_id == player => match self.fleet.ensure(ship_id) {
                Some(idx) => idx,
                None => return,
            },
            None => return,
        };
        let Some(hold) = self.fleet.holds[idx].as_mut() else {
            return;
        };
        if hold.authority != Some(player) {
            return;
        }
        let next_seq = self.seq + 1;
        let Some(star) = hold.recent.keep(record, next_seq) else {
            return;
        };
        self.seq = next_seq;
        if let Some(seat) = self.seats[slot].as_mut() {
            seat.star = Some(star);
        }
    }

    /// Handles a player request to board or leave a ship.
    pub(super) fn ship_act(&mut self, conn: ConnId, act: ShipAct, out: &mut Outbox) {
        let Some(player) = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_ref())
            .map(|seat| seat.info.id)
        else {
            return;
        };
        match act {
            ShipAct::Board(ship_id) => self.board_ship(conn, player, ship_id, out),
            ShipAct::Leave(ship_id) => self.leave_ship(player, ship_id, out),
        }
    }

    /// Grants `player` the pilot seat and authority on `ship_id` unless another player flies it.
    fn board_ship(&mut self, conn: ConnId, player: PlayerId, ship_id: PlayerId, out: &mut Outbox) {
        let idx = match self.fleet.slot_of(ship_id) {
            Some(idx) => idx,
            None if ship_id == player => match self.fleet.ensure(ship_id) {
                Some(idx) => idx,
                None => return,
            },
            None => {
                out.send(conn, ServerMsg::Ship(ShipEvent::Denied(ship_id)));
                return;
            }
        };
        if self.fleet.holds[idx]
            .as_ref()
            .is_some_and(|h| h.pilot.is_some() && h.pilot != Some(player))
        {
            out.send(conn, ServerMsg::Ship(ShipEvent::Denied(ship_id)));
            return;
        }
        self.release_held(player, ship_id, out);
        let Some(hold) = self.fleet.holds[idx].as_mut() else {
            return;
        };
        hold.pilot = Some(player);
        hold.authority = Some(player);
        let msg = ServerMsg::Ship(ShipEvent::Boarded {
            ship: ship_id,
            pilot: player,
        });
        self.broadcast(&msg, out);
    }

    /// Releases `player` from the pilot seat of `ship_id` while leaving its authority parked on it.
    fn leave_ship(&mut self, player: PlayerId, ship_id: PlayerId, out: &mut Outbox) {
        let Some(idx) = self.fleet.slot_of(ship_id) else {
            return;
        };
        let Some(hold) = self.fleet.holds[idx].as_mut() else {
            return;
        };
        if hold.pilot != Some(player) {
            return;
        }
        hold.pilot = None;
        let gone = hold.owner_gone.then_some(ship_id);
        let left = ServerMsg::Ship(ShipEvent::Left {
            ship: ship_id,
            pilot: player,
        });
        self.broadcast(&left, out);
        if let Some(gone_id) = gone {
            self.clear_ship_slot(idx);
            self.broadcast(&ServerMsg::Ship(ShipEvent::Gone(gone_id)), out);
        }
    }

    /// Cleans up ship seats and authorities when `leaving` disconnects, keeping their ship alive
    /// while another player pilots it.
    pub(super) fn leave_ships(&mut self, leaving: PlayerId, out: &mut Outbox) {
        for idx in 0..self.fleet.holds.len() {
            let Some(hold) = self.fleet.holds[idx].as_mut() else {
                continue;
            };
            let ship_id = hold.id;
            let unpilot = hold.pilot == Some(leaving);
            if unpilot {
                hold.pilot = None;
            }
            if hold.authority == Some(leaving) {
                hold.authority = None;
            }
            let drop_unpiloted = ship_id == leaving && hold.pilot.is_none();
            let remove_abandoned = unpilot && hold.owner_gone;
            if ship_id == leaving && !drop_unpiloted {
                hold.owner_gone = true;
            }
            if unpilot {
                let left = ServerMsg::Ship(ShipEvent::Left {
                    ship: ship_id,
                    pilot: leaving,
                });
                self.broadcast(&left, out);
            }
            if drop_unpiloted || remove_abandoned {
                self.clear_ship_slot(idx);
                self.broadcast(&ServerMsg::Ship(ShipEvent::Gone(ship_id)), out);
            }
        }
    }
}
