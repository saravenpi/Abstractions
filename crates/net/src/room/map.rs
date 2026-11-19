//! What players share for the map: the ground they explored on each body and one waypoint
//! each.
//!
//! The room merges explored cells into one bitset per body and tells everyone only the new
//! cells, at most once a second per body. A waypoint lives as long as its player's seat.

use std::collections::HashMap;
use std::mem;

use glam::Vec3;

use super::Room;
use crate::host::{ConnId, Outbox};
use crate::ids::BodyId;
use crate::wire::{EXPLORED_COLUMNS, EXPLORED_ROWS, ServerMsg, Waypoint, clean_name};

/// Words in the explored bitset of one body: one bit per cell.
pub const EXPLORED_WORDS: usize = (EXPLORED_COLUMNS * EXPLORED_ROWS / 64) as usize;

/// Most bodies the room keeps explored ground for, so the welcome stays under
/// [`crate::wire::MESSAGE_MAX`]. Cells of further bodies are dropped.
const BODIES_MAX: usize = 128;

/// Seconds between two broadcasts of new cells of one body.
const EXPLORED_EVERY: f64 = 1.0;

/// The explored ground of one body.
#[derive(Debug)]
pub(super) struct Ground {
    /// One bit per cell, cell `n` in bit `n % 64` of word `n / 64`.
    bits: Vec<u64>,
    /// Cells explored since the last broadcast, in the order they came.
    fresh: Vec<u32>,
    /// Network time of the last broadcast.
    sent: f64,
}

/// Returns the ground of `world`, leaving out bitsets of the wrong size.
pub(super) fn grounds(explored: Vec<(BodyId, Vec<u64>)>) -> HashMap<BodyId, Ground> {
    let sized = explored
        .into_iter()
        .filter(|(_, bits)| bits.len() == EXPLORED_WORDS);
    sized.map(|(body, bits)| (body, ground(bits))).collect()
}

/// Returns the ground explored as `bits`, with nothing new to tell.
fn ground(bits: Vec<u64>) -> Ground {
    Ground {
        bits,
        fresh: Vec::new(),
        sent: f64::NEG_INFINITY,
    }
}

impl Room {
    /// Merges the `cells` the player on `conn` explored on `body`. Cells out of the grid are
    /// dropped, and so is a new body once [`BODIES_MAX`] are known.
    pub(super) fn explored(&mut self, conn: ConnId, body: BodyId, cells: &[u32]) {
        let full = self.explored.len() >= BODIES_MAX;
        if self.seat_of(conn).is_none() || (full && !self.explored.contains_key(&body)) {
            return;
        }
        let blank = || ground(vec![0; EXPLORED_WORDS]);
        let ground = self.explored.entry(body).or_insert_with(blank);
        for &cell in cells {
            let Some(word) = ground.bits.get_mut(cell as usize / 64) else {
                continue;
            };
            let bit = 1 << (cell % 64);
            if *word & bit == 0 {
                *word |= bit;
                ground.fresh.push(cell);
            }
        }
    }

    /// Tells everyone the new cells of each body whose last broadcast is a second old.
    pub(super) fn spread_explored(&mut self, now: f64, out: &mut Outbox) {
        let seats = &self.seats;
        for (body, ground) in &mut self.explored {
            if ground.fresh.is_empty() || now - ground.sent < EXPLORED_EVERY {
                continue;
            }
            ground.sent = now;
            let cells = mem::take(&mut ground.fresh);
            let msg = ServerMsg::Explored { body: *body, cells };
            for seat in seats.iter().flatten() {
                out.send(seat.conn, msg.clone());
            }
        }
    }

    /// Sets or clears the waypoint of the player on `conn` and tells the others. A waypoint
    /// whose direction is not a finite, non-zero vector is ignored.
    pub(super) fn waypoint(&mut self, conn: ConnId, waypoint: Option<Waypoint>, out: &mut Outbox) {
        let Some(seat) = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_mut())
        else {
            return;
        };
        let waypoint = match waypoint.map(cleaned) {
            Some(None) => return,
            kept => kept.flatten(),
        };
        seat.waypoint.clone_from(&waypoint);
        let msg = ServerMsg::Waypoint {
            player: seat.info.id,
            waypoint,
        };
        self.send_others(conn, &msg, out);
    }

    /// Returns the explored ground of every body, as the welcome and the world file carry it.
    pub(super) fn explored_list(&self) -> Vec<(BodyId, Vec<u64>)> {
        let all = self.explored.iter();
        all.map(|(body, ground)| (*body, ground.bits.clone()))
            .collect()
    }

    /// Returns the live waypoint of every player in the room.
    pub(super) fn waypoints(&self) -> Vec<(crate::log::PlayerId, Waypoint)> {
        let seats = self.seats.iter().flatten();
        seats
            .filter_map(|seat| Some((seat.info.id, seat.waypoint.clone()?)))
            .collect()
    }
}

/// Returns `waypoint` with its direction made unit and its name cleaned, or nothing when the
/// direction has no length.
fn cleaned(waypoint: Waypoint) -> Option<Waypoint> {
    let dir = Vec3::from_array(waypoint.dir).try_normalize()?;
    Some(Waypoint {
        dir: dir.to_array(),
        name: clean_name(&waypoint.name),
        ..waypoint
    })
}
