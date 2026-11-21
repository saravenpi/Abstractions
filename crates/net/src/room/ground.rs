//! What a joined player puts on the ground and takes off it. The room is the authority: it gives
//! each thing its id, tells everyone where it lies, and grants a take to the first who asks.

use super::Room;
use crate::ground::Placed;
use crate::host::{ConnId, Outbox};
use crate::log::PlayerId;
use crate::wire::{Event, GROUND_CHUNK, Lying, ServerMsg};

impl Room {
    /// Puts the thing the player `by` dropped at universe time `clock` on the ground and tells
    /// everyone, the player too. A thing that is not sane, or that the ground cannot take, is
    /// handed back to the player alone with [`Event::Kept`].
    pub(super) fn dropped(
        &mut self,
        conn: ConnId,
        (by, clock): (PlayerId, f64),
        lying: Lying,
        out: &mut Outbox,
    ) {
        let tag = lying.tag;
        let lying = Lying {
            id: 0,
            by,
            since: clock,
            ..lying
        };
        let placed = match lying.sane() {
            true => self.ground.lay(lying),
            false => Placed::Refused,
        };
        match placed {
            Placed::Laid(lying) | Placed::Joined(lying) => {
                self.dirty = true;
                self.broadcast(&ServerMsg::Hunt(Event::Lies(lying)), out);
            }
            Placed::Refused => out.send(conn, ServerMsg::Hunt(Event::Kept(tag))),
        }
    }

    /// Gives the thing `id`, or `mass` kilograms of its heap, to the player `by` who asks first,
    /// and tells everyone what is left. A player who asks for a thing that is gone hears
    /// [`Event::Denied`].
    pub(super) fn picked(
        &mut self,
        conn: ConnId,
        by: PlayerId,
        (id, mass): (u64, f32),
        out: &mut Outbox,
    ) {
        let Some(took) = self.ground.take(id, mass) else {
            out.send(conn, ServerMsg::Hunt(Event::Denied(id)));
            return;
        };
        self.dirty = true;
        out.send(conn, ServerMsg::Hunt(Event::Granted(took.got)));
        let after = match took.left {
            Some(left) => Event::Lies(Lying { tag: 0, ..left }),
            None => Event::Gone { id, by },
        };
        self.broadcast(&ServerMsg::Hunt(after), out);
    }

    /// Tells the player on `conn` every thing on the ground, [`GROUND_CHUNK`] to a message.
    pub(in crate::room) fn tell_ground(&self, conn: ConnId, out: &mut Outbox) {
        for chunk in self.ground.lying.chunks(GROUND_CHUNK) {
            out.send(conn, ServerMsg::Hunt(Event::Ground(chunk.to_vec())));
        }
    }
}
