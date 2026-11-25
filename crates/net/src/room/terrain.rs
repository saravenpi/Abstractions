//! Felled trees, terrain digs, and ephemeral gunfire shots handled by the room.

use super::Room;
use crate::chest::valid_dir;
use crate::host::{ConnId, Outbox};
use crate::wire::{
    DIG_CHUNK, DIGS_MAX, DigOp, ServerMsg, Shot, TREE_CHUNK, TREE_ID_MAX, TREES_MAX, TreeFell,
};

impl Room {
    /// Records a felled tree once per `(body, id)` within limits and tells the other players.
    pub(super) fn tree_fell(&mut self, conn: ConnId, tree: TreeFell, out: &mut Outbox) {
        let valid = !tree.id.is_empty() && tree.id.len() <= TREE_ID_MAX;
        if self.seat_of(conn).is_none() || !valid || self.props.trees.len() >= TREES_MAX {
            return;
        }
        if !self.props.tree_seen.insert((tree.body, tree.id.clone())) {
            return;
        }
        self.props.trees.push(tree.clone());
        self.dirty = true;
        self.send_others(conn, &ServerMsg::Tree(tree), out);
    }

    /// Records a valid terrain crater dig within limits and tells the other players.
    pub(super) fn dig(&mut self, conn: ConnId, dig: DigOp, out: &mut Outbox) {
        let valid = valid_dir(dig.dir)
            && dig.radius.is_finite()
            && dig.radius > 0.0
            && dig.depth.is_finite()
            && dig.depth > 0.0;
        if self.seat_of(conn).is_none() || !valid || self.props.digs.len() >= DIGS_MAX {
            return;
        }
        self.props.digs.push(dig);
        self.dirty = true;
        self.send_others(conn, &ServerMsg::Dig(dig), out);
    }

    /// Relays a valid firearm shot from `conn` to all other joined players without storing it.
    pub(super) fn shot(&mut self, conn: ConnId, shot: Shot, out: &mut Outbox) {
        let Some(by) = self.player_id(conn) else {
            return;
        };
        let valid = shot.origin.iter().all(|v| v.is_finite()) && valid_dir(shot.dir);
        if valid {
            self.send_others(conn, &ServerMsg::Shot { by, shot }, out);
        }
    }

    /// Sends felled trees to `conn` in chunks of [`TREE_CHUNK`].
    pub(in crate::room) fn tell_trees(&self, conn: ConnId, out: &mut Outbox) {
        for chunk in self.props.trees.chunks(TREE_CHUNK) {
            out.send(conn, ServerMsg::Trees(chunk.to_vec()));
        }
    }

    /// Sends terrain digs to `conn` in chunks of [`DIG_CHUNK`].
    pub(in crate::room) fn tell_digs(&self, conn: ConnId, out: &mut Outbox) {
        for chunk in self.props.digs.chunks(DIG_CHUNK) {
            out.send(conn, ServerMsg::Digs(chunk.to_vec()));
        }
    }
}
