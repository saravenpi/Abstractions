//! Shared chests, torches, felled trees, and terrain digs kept by the room and synced to
//! players on join and on every authoritative edit.

use std::collections::HashSet;

use super::Room;
use crate::chest::{ChestItem, Chests, PlaceableKind};
use crate::host::{ConnId, Outbox};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::store::World;
use crate::wire::{
    CHEST_CHUNK, ChestAct, ChestEvent, DIGS_MAX, DigOp, ServerMsg, TREES_MAX, TreeFell,
};

/// Persistent world props and terrain edits held in memory by a [`Room`].
#[derive(Debug, Default)]
pub(super) struct PropState {
    /// Placed chests and torches on bodies.
    pub(super) chests: Chests,
    /// Felled trees recorded in order for join snapshots and saves.
    pub(super) trees: Vec<TreeFell>,
    /// Deduplication index of felled `(body, tree_id)` pairs.
    pub(super) tree_seen: HashSet<(BodyId, String)>,
    /// Terrain crater digs recorded in order for join snapshots and saves.
    pub(super) digs: Vec<DigOp>,
}

impl PropState {
    /// Moves chests, trees, and digs out of `world` into bounded room state.
    pub(super) fn from_world(world: &mut World) -> Self {
        let saved = std::mem::take(&mut world.chests);
        let mut chests = Chests::new(saved.props);
        chests.next_id = chests.next_id.max(saved.next_id);
        let mut tree_seen = HashSet::new();
        let mut trees = Vec::new();
        for tree in std::mem::take(&mut world.trees) {
            if trees.len() >= TREES_MAX {
                break;
            }
            if tree_seen.insert((tree.body, tree.id.clone())) {
                trees.push(tree);
            }
        }
        let mut digs = std::mem::take(&mut world.digs);
        digs.truncate(DIGS_MAX);
        Self {
            chests,
            trees,
            tree_seen,
            digs,
        }
    }
}

impl Room {
    /// Returns the joined player id for `conn`, if seated.
    pub(super) fn player_id(&self, conn: ConnId) -> Option<PlayerId> {
        self.seat_of(conn)
            .and_then(|slot| self.seats[slot].as_ref())
            .map(|seat| seat.info.id)
    }

    /// Handles an authoritative chest or torch action from `conn` and broadcasts accepted changes.
    pub(super) fn chest_act(&mut self, conn: ConnId, act: ChestAct, out: &mut Outbox) {
        let Some(by) = self.player_id(conn) else {
            return;
        };
        match act {
            ChestAct::Place {
                body,
                dir,
                yaw,
                kind,
            } => self.place_prop(body, (dir, yaw), kind, out),
            ChestAct::Break { id } => self.break_prop(by, id, out),
            ChestAct::SetSlot { id, slot, item } => self.set_prop_slot(id, slot, item, out),
            ChestAct::Sort { id } => self.sort_prop(id, out),
        }
    }

    /// Places a new chest or torch and broadcasts [`ChestEvent::Placed`] to everyone.
    fn place_prop(
        &mut self,
        body: BodyId,
        (dir, yaw): ([f32; 3], f32),
        kind: PlaceableKind,
        out: &mut Outbox,
    ) {
        let Some(prop) = self.props.chests.place(body, dir, yaw, kind) else {
            return;
        };
        self.dirty = true;
        self.broadcast(&ServerMsg::Chest(ChestEvent::Placed(prop)), out);
    }

    /// Removes placed prop `id` and broadcasts [`ChestEvent::Broken`] to everyone.
    fn break_prop(&mut self, by: PlayerId, id: u64, out: &mut Outbox) {
        if self.props.chests.remove(id).is_none() {
            return;
        }
        self.dirty = true;
        self.broadcast(&ServerMsg::Chest(ChestEvent::Broken { id, by }), out);
    }

    /// Updates slot `slot` of chest `id` and broadcasts [`ChestEvent::Slot`] to everyone.
    fn set_prop_slot(&mut self, id: u64, slot: u8, item: Option<ChestItem>, out: &mut Outbox) {
        if !self.props.chests.set_slot(id, slot, item.clone()) {
            return;
        }
        self.dirty = true;
        let msg = ServerMsg::Chest(ChestEvent::Slot { id, slot, item });
        self.broadcast(&msg, out);
    }

    /// Sorts chest `id` and broadcasts [`ChestEvent::Sorted`] with the updated slots.
    fn sort_prop(&mut self, id: u64, out: &mut Outbox) {
        if !self.props.chests.sort_slots(id) {
            return;
        }
        let Some(slots) = self.props.chests.find(id).map(|p| p.slots.clone()) else {
            return;
        };
        self.dirty = true;
        self.broadcast(&ServerMsg::Chest(ChestEvent::Sorted { id, slots }), out);
    }

    /// Sends placed chests and torches to `conn` in chunks of [`CHEST_CHUNK`].
    pub(in crate::room) fn tell_chests(&self, conn: ConnId, out: &mut Outbox) {
        for chunk in self.props.chests.props.chunks(CHEST_CHUNK) {
            out.send(conn, ServerMsg::Chests(chunk.to_vec()));
        }
    }
}
