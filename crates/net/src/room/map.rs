//! What players share for the map: the ground they explored on each body and one waypoint
//! each.
//!
//! The room merges explored leaves into one set per body and tells everyone only the new
//! leaves, at most once a second per body. A waypoint lives as long as its player's seat.

use std::collections::HashMap;
use std::mem;

use glam::Vec3;

use super::Room;
use crate::host::{ConnId, Outbox};
use crate::ids::BodyId;
use crate::leaves::{Leaves, Run};
use crate::wire::{ServerMsg, Waypoint, clean_name};

/// Most bodies the room keeps explored ground for, so the welcome stays under
/// [`crate::wire::MESSAGE_MAX`]. Leaves of further bodies are dropped.
pub const BODIES_MAX: usize = 128;

/// Most runs the room keeps for one body, so a welcome of [`BODIES_MAX`] bodies stays under
/// [`crate::wire::MESSAGE_MAX`]. Leaves past it are dropped.
pub const RUNS_MAX: usize = 6144;

/// Seconds between two broadcasts of new leaves of one body.
const EXPLORED_EVERY: f64 = 1.0;

/// The explored ground of one body.
#[derive(Debug)]
pub(super) struct Ground {
    /// Every leaf explored.
    leaves: Leaves,
    /// Leaves explored since the last broadcast.
    fresh: Leaves,
    /// Network time of the last broadcast.
    sent: f64,
}

/// Returns the ground of `world`, leaving out bodies past [`BODIES_MAX`].
pub(super) fn grounds(explored: Vec<(BodyId, Vec<Run>)>) -> HashMap<BodyId, Ground> {
    let kept = explored.into_iter().take(BODIES_MAX);
    kept.map(|(body, runs)| (body, ground(Leaves::from_runs(&runs))))
        .collect()
}

/// Returns the ground explored as `leaves`, with nothing new to tell.
fn ground(leaves: Leaves) -> Ground {
    Ground {
        leaves,
        fresh: Leaves::default(),
        sent: f64::NEG_INFINITY,
    }
}

impl Room {
    /// Merges the leaves in `runs` that the player on `conn` explored on `body`. Keys that name
    /// no leaf are dropped, and so is a new body once [`BODIES_MAX`] are known or a body that
    /// holds [`RUNS_MAX`] runs.
    pub(super) fn explored(&mut self, conn: ConnId, body: BodyId, runs: &[Run]) {
        let full = self.explored.len() >= BODIES_MAX;
        if self.seat_of(conn).is_none() || (full && !self.explored.contains_key(&body)) {
            return;
        }
        let ground = self
            .explored
            .entry(body)
            .or_insert_with(|| ground(Leaves::default()));
        if ground.leaves.run_count() >= RUNS_MAX {
            return;
        }
        let new = ground.leaves.merge(&Leaves::from_runs(runs));
        ground.fresh.merge(&new);
    }

    /// Tells everyone the new leaves of each body whose last broadcast is a second old.
    pub(super) fn spread_explored(&mut self, now: f64, out: &mut Outbox) {
        let seats = &self.seats;
        for (body, ground) in &mut self.explored {
            if ground.fresh.is_empty() || now - ground.sent < EXPLORED_EVERY {
                continue;
            }
            ground.sent = now;
            let runs = mem::take(&mut ground.fresh).runs();
            let msg = ServerMsg::Explored { body: *body, runs };
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
    pub(super) fn explored_list(&self) -> Vec<(BodyId, Vec<Run>)> {
        let all = self.explored.iter();
        all.map(|(body, ground)| (*body, ground.leaves.runs()))
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
