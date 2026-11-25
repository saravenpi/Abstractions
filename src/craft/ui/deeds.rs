//! The deeds of the screen, one place for the keys and the pointer: choosing a row, an intent or a
//! card, holding and slotting a tool, and putting a tool or a stack of the pack down on the ground.
//! Making and treating are in `make`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Workshop;
use super::bench::{Bench, Pane};
use super::click::Press;
use super::here::Here;
use super::lines::{Line, What, lines};
use crate::craft::ground::{Shed, shed_item, shed_stack};
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
    /// What the pack lets go of, to be put down on the ground.
    pub(super) shed: ResMut<'w, Shed>,
    /// The keys held, for Shift.
    pub(super) keys: Res<'w, ButtonInput<KeyCode>>,
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
            Press::Pane(pane) => self.bench.pane = pane,
            Press::GridSlot(at) => self.click_grid_slot(at),
            Press::SortPack => self.sort_pack(),
            Press::Category(cat) => self.pick_category(cat),
            Press::Recipe(idx) => self.bench.recipe_idx = idx,
            Press::CraftBatch(n) => self.craft_batch(n),
            Press::CycleMineral => self.cycle_mineral(),
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
            self.pack.put_bar(held, id);
        }
    }

    /// Puts the selected tool or stack down on the ground, half the stack with Shift held, and keeps
    /// the cursor on the list. Nothing is lost: a tool lies on the ground until it is taken back.
    fn drop_selected(&mut self) {
        let half = self
            .keys
            .any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        if self.drop_grid_or_row(half) {
            return;
        }
        match self.selected() {
            Some(What::Item(id)) => shed_item(&mut self.pack, &mut self.shed, id),
            Some(What::Stack(at)) => {
                let share = [1.0, 0.5][usize::from(half)];
                shed_stack(&mut self.pack, &mut self.shed, at, share);
            }
            None => return,
        }
        let last = self.lines().len().saturating_sub(1);
        self.bench.cursor = self.bench.cursor.min(last);
    }

    /// Puts a tool in hotbar slot `slot` (or finishes a picked grid swap into `slot`).
    fn slot(&mut self, slot: usize) {
        if let Some(from) = self.bench.picked_slot.take() {
            self.pack.swap_slots(from, slot);
            self.bench.selected_slot = Some(slot);
            return;
        }
        let tool = match self.bench.pane {
            Pane::Pack => match self.selected() {
                Some(What::Item(id)) => Some(id),
                _ => None,
            },
            Pane::Workshop => self.bench.made,
        };
        match tool.filter(|id| self.pack.item(*id).is_some()) {
            Some(id) => {
                self.pack.put_bar(slot, id);
                self.bench.note = format!("In slot {}: press {} to hold it.", slot + 1, slot + 1);
            }
            None => self.pack.bar.select(slot),
        }
    }
}
