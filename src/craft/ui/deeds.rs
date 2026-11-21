//! The deeds of the screen, one place for the keys and the pointer: choosing a row, an intent or a
//! card, holding, slotting and dropping from the pack. Making and treating are in `make`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Workshop;
use super::bench::{Bench, Pane};
use super::click::Press;
use super::here::Here;
use super::lines::{Line, What, lines};
use crate::craft::pack::Pack;

/// What a deed reads and changes: the pack, the bench, the world underfoot and the screen state.
#[derive(SystemParam)]
pub struct Deed<'w> {
    /// The pack.
    pub pack: ResMut<'w, Pack>,
    /// The state of the screen.
    pub bench: ResMut<'w, Bench>,
    /// The world underfoot.
    pub here: Here<'w>,
    /// Where the screen goes next.
    next: ResMut<'w, NextState<Workshop>>,
}

impl Deed<'_> {
    /// Returns the rows of the pack, in the order the pane shows them.
    pub fn lines(&self) -> Vec<Line> {
        lines(&self.pack, &self.here.place(), self.here.life())
    }

    /// Returns what the cursor of the pack stands on.
    fn selected(&self) -> Option<What> {
        self.lines().get(self.bench.cursor).map(|line| line.what)
    }

    /// Runs `press`.
    pub fn press(&mut self, press: Press) {
        let (intent, pick) = (self.bench.intent, self.bench.pick);
        match press {
            Press::Row(at) => (self.bench.pane, self.bench.cursor) = (Pane::Pack, at),
            Press::Intent(intent) => self.ask(intent, pick),
            Press::Card(at) => self.ask(intent, at),
            Press::Hold => self.hold(),
            Press::Drop => self.drop_selected(),
            Press::Make => self.make(),
            Press::Head => self.bench.swap[1] += 1,
            Press::Shaft => self.bench.swap[0] += 1,
            Press::Treat => self.treat(),
            Press::Slot(slot) => self.slot(slot),
            Press::Close => self.next.set(Workshop::Closed),
        }
    }

    /// Asks the workshop for `intent` and focuses its card at `pick`, with no swap.
    pub fn ask(&mut self, intent: crate::craft::build::Intent, pick: usize) {
        let bench = &mut self.bench;
        if (bench.pane, bench.intent, bench.pick) == (Pane::Workshop, intent, pick) {
            return;
        }
        (bench.pane, bench.intent, bench.pick) = (Pane::Workshop, intent, pick.min(2));
        (bench.swap, bench.note) = ([0, 0], String::new());
    }

    /// Puts the selected tool in the held slot.
    fn hold(&mut self) {
        if let Some(What::Item(id)) = self.selected() {
            let held = self.pack.bar.held;
            self.pack.bar.put(held, id);
        }
    }

    /// Drops the selected entry of the pack, and keeps the cursor on the list.
    fn drop_selected(&mut self) {
        match self.selected() {
            Some(What::Item(id)) => {
                self.pack.bar.forget(id);
                self.pack.items.retain(|item| item.id != id);
            }
            Some(What::Stack(at)) => _ = self.pack.stacks.remove(at),
            None => return,
        }
        let last = self.lines().len().saturating_sub(1);
        self.bench.cursor = self.bench.cursor.min(last);
    }

    /// Puts a tool in hotbar slot `slot`: the selected tool in the pack pane, the last made tool in
    /// the workshop pane. With no such tool, holds the slot.
    fn slot(&mut self, slot: usize) {
        let tool = match self.bench.pane {
            Pane::Pack => match self.selected() {
                Some(What::Item(id)) => Some(id),
                _ => None,
            },
            Pane::Workshop => self.bench.made,
        };
        match tool.filter(|id| self.pack.item(*id).is_some()) {
            Some(id) => {
                self.pack.bar.put(slot, id);
                self.bench.note = format!("In slot {}: press {} to hold it.", slot + 1, slot + 1);
            }
            None => self.pack.bar.select(slot),
        }
    }
}
