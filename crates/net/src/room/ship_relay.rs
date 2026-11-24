//! Shared ship state sync on join and zero-allocation ship relay ticks.

use super::Room;
use super::relay::{due, offer, offer_record, owed};
use super::seat::Sent;
use crate::host::{ConnId, Outbox};
use crate::ids::StarId;
use crate::log::PlayerId;
use crate::wire::{Record, ServerMsg, ShipEvent, ShipPilot};

/// Seconds between heartbeat records resent for an unattended parked ship.
const PARKED_EVERY: f64 = 2.0;

impl Room {
    /// Sends active ship-to-pilot bindings to `conn` upon joining when any ship is piloted.
    pub(super) fn tell_ships(&self, conn: ConnId, out: &mut Outbox) {
        let pilots: Vec<ShipPilot> = self
            .fleet
            .holds
            .iter()
            .flatten()
            .filter_map(|hold| {
                hold.pilot.map(|pilot| ShipPilot {
                    ship: hold.id,
                    pilot,
                })
            })
            .collect();
        if !pilots.is_empty() {
            out.send(conn, ServerMsg::Ship(ShipEvent::World(pilots)));
        }
    }

    /// Marks `id`'s ship back under active ownership if another player held it while away.
    pub(super) fn rejoin_ship(&mut self, id: PlayerId) {
        if let Some(idx) = self.fleet.slot_of(id)
            && let Some(hold) = self.fleet.holds[idx].as_mut()
        {
            hold.owner_gone = false;
        }
    }

    /// Releases any other ship piloted by `player` and clears `authority` on other ships they held.
    pub(super) fn release_held(&mut self, player: PlayerId, target: PlayerId, out: &mut Outbox) {
        for idx in 0..self.fleet.holds.len() {
            let Some(hold) = self.fleet.holds[idx].as_mut() else {
                continue;
            };
            if hold.id == target {
                continue;
            }
            if hold.pilot == Some(player) {
                hold.pilot = None;
                let (old_id, remove) = (hold.id, hold.owner_gone);
                let left = ServerMsg::Ship(ShipEvent::Left {
                    ship: old_id,
                    pilot: player,
                });
                self.broadcast(&left, out);
                if remove {
                    self.clear_ship_slot(idx);
                    self.broadcast(&ServerMsg::Ship(ShipEvent::Gone(old_id)), out);
                }
            } else if hold.authority == Some(player) {
                hold.authority = None;
            }
        }
    }

    /// Removes the hold at slot `idx`.
    pub(super) fn clear_ship_slot(&mut self, idx: usize) {
        self.fleet.holds[idx] = None;
    }

    /// Relays active and parked ship records to recipient `to` into `self.batch` without allocating.
    pub(super) fn relay_ships(
        &mut self,
        to: usize,
        (conn, me, star): (ConnId, PlayerId, Option<StarId>),
        now: f64,
        out: &mut Outbox,
    ) {
        let clock = self.clock(now);
        for (idx, hold) in self.fleet.holds.iter().enumerate() {
            let Some(hold) = hold.as_ref().filter(|h| h.authority != Some(me)) else {
                continue;
            };
            let Some(newest) = hold.recent.newest() else {
                continue;
            };
            let sent = &mut self.fleet.sent[to][idx];
            if hold.authority.is_none() {
                if newest.seq > sent.seq || now - sent.at >= PARKED_EVERY {
                    let mut head = newest.head;
                    head.time = clock;
                    let rec = Record {
                        head,
                        body: &newest.body,
                    };
                    offer_record(&mut self.batch, rec, conn, out);
                    *sent = Sent {
                        seq: newest.seq,
                        at: now,
                    };
                }
                continue;
            }
            if !due(newest, sent, star, now) {
                continue;
            }
            let near = star == Some(newest.star);
            let since = sent.seq.max(self.relayed);
            for latest in owed(&hold.recent, newest, since, near) {
                offer(&mut self.batch, latest, conn, out);
            }
            *sent = Sent {
                seq: newest.seq,
                at: now,
            };
        }
    }
}
