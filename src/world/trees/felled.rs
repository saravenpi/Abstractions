//! Tree chop hit tracking and compact text serialization for saves.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;

use super::TreeId;

/// Chop hits required to fell a tree barehanded (1 damage per hit).
pub const TREE_CHOPS: u8 = 3;

/// Damage and felling state of every damaged or felled tree in the world.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Felled(Arc<HashMap<TreeId, u8>>);

impl Felled {
    /// Returns the number of chop hits sustained by `id`.
    pub fn hits(&self, id: TreeId) -> u8 {
        self.0.get(&id).copied().unwrap_or(0)
    }

    /// Returns true when `id` has been completely felled.
    pub fn gone(&self, id: TreeId) -> bool {
        self.hits(id) >= TREE_CHOPS
    }

    /// Applies `power` chop damage to `id`, returning true if the tree is now felled.
    pub fn chop(&mut self, id: TreeId, power: u8) -> bool {
        let entry = Arc::make_mut(&mut self.0).entry(id).or_insert(0);
        *entry = (*entry).saturating_add(power.max(1)).min(TREE_CHOPS);
        *entry >= TREE_CHOPS
    }

    /// Marks `id` as completely felled regardless of prior hit count.
    pub fn mark_gone(&mut self, id: TreeId) {
        Arc::make_mut(&mut self.0).insert(id, TREE_CHOPS);
    }

    /// Iterates every completely felled tree id.
    pub fn felled_ids(&self) -> impl Iterator<Item = TreeId> + '_ {
        self.0
            .iter()
            .filter_map(|(&id, &hits)| (hits >= TREE_CHOPS).then_some(id))
    }

    /// Encodes damaged and felled trees deterministically for world save storage.
    pub fn to_text(&self) -> Vec<String> {
        let mut rows: Vec<(TreeId, u8)> = self.0.iter().map(|(&id, &h)| (id, h)).collect();
        rows.sort_unstable_by_key(|(id, _)| *id);
        rows.into_iter()
            .map(|(id, h)| format!("{}:{h}", id.text()))
            .collect()
    }

    /// Restores tree chop states from saved text rows.
    pub fn from_text(text: &[String]) -> Self {
        let map = text
            .iter()
            .filter_map(|row| {
                let mut parts = row.split(':');
                let id = TreeId::parse(parts.next()?)?;
                let hits = parts
                    .next()
                    .and_then(|s| s.parse::<u8>().ok())
                    .unwrap_or(TREE_CHOPS)
                    .min(TREE_CHOPS);
                Some((id, hits))
            })
            .collect();
        Self(Arc::new(map))
    }
}
