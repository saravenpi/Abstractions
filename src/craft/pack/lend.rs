//! Lending a tool out of the pack: a thrown spear leaves the pack while it is in the world and
//! comes back to its hotbar slot when it is picked up. The pack keeps a record of every tool that
//! is out and of where it last came to rest, and the record is saved with the pack, so a tool whose
//! spear leaves the world (the animal it stood in is gone, the world is closed) is never lost: it
//! lies where the record says.

use bevy::math::DVec3;
use serde::{Deserialize, Serialize};

use super::{Item, Pack, SlotEntry, TOTAL_SLOTS};
use crate::universe::BodyId;

/// A tool taken out of the pack, with the hotbar slot it stood in.
pub type Lent = (Item, Option<usize>);

/// A tool of the pack that is out in the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Out {
    /// The tool.
    pub item: Item,
    /// The hotbar slot it stood in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<usize>,
    /// The planet it came to rest on, or nothing while it has not come to rest anywhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyId>,
    /// Where it rests, in metres from the centre of that planet in its turning frame.
    pub at: [f64; 3],
}

impl Pack {
    /// Takes the tool with `id` out of the pack and its slot, notes it as out in the world, and
    /// returns it with that slot.
    pub fn lend(&mut self, id: u32) -> Option<Lent> {
        self.ensure_grid();
        self.sync_bar_into_grid();
        let at = self.items.iter().position(|item| item.id == id)?;
        let slot = self
            .grid
            .iter()
            .position(|s| matches!(s, Some(SlotEntry::Tool(has)) if *has == id));
        if let Some(s) = slot {
            self.grid[s] = None;
        }
        let item = self.items.remove(at);
        self.sync_views();
        self.out.push(Out {
            item: item.clone(),
            slot,
            body: None,
            at: [0.0; 3],
        });
        Some((item, slot))
    }

    /// Puts a lent tool back, in the slot it stood in when empty, else in a free slot.
    pub fn restore(&mut self, (item, slot): Lent) {
        self.ensure_grid();
        let id = item.id;
        self.out.retain(|out| out.item.id != id);
        self.items.push(item);
        match slot.filter(|s| *s < TOTAL_SLOTS && self.grid[*s].is_none()) {
            Some(slot) => self.grid[slot] = Some(SlotEntry::Tool(id)),
            None => {
                if let Some(free) = self.grid.iter_mut().find(|s| s.is_none()) {
                    *free = Some(SlotEntry::Tool(id));
                }
            }
        }
        self.sync_views();
    }

    /// Notes that the tool with `id` that is out rests on `body` at `at`. Returns false, changing
    /// nothing, when the record already says so within `slack` metres or no such tool is out.
    pub fn rest(&mut self, id: u32, (body, at): (BodyId, DVec3), slack: f64) -> bool {
        let Some(out) = self.out.iter_mut().find(|out| out.item.id == id) else {
            return false;
        };
        let same = out.body == Some(body) && DVec3::from(out.at).distance(at) <= slack;
        if !same {
            (out.body, out.at) = (Some(body), at.to_array());
        }
        !same
    }
}
