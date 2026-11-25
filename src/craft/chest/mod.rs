//! Placeable shared chests, flickering torches, and the dual-grid chest storage overlay.

pub mod codec;
mod open;
mod place;
mod save;
mod show;
#[cfg(test)]
mod tests;
mod ui;
mod ui_ops;

use std::time::Duration;

use abstractions_net::chest::Chests;
use abstractions_net::wire::ChestAct;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::craft::pack::Pack;
use crate::saves::ActiveWorld;
use crate::screens::in_control;
use crate::space::{Anchor, on_foot};

/// Interval in seconds between proximity sync sweeps for placed chests and torches.
const SYNC_STEP: f32 = 0.25;

/// Whether the dual-grid chest storage overlay is open.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChestScreen {
    /// No chest overlay is open.
    #[default]
    Closed,
    /// A placed chest overlay is open.
    Open,
}

/// Run condition: true while the chest storage overlay is open.
pub fn shown(state: Option<Res<State<ChestScreen>>>) -> bool {
    state.is_some_and(|state| *state.get() == ChestScreen::Open)
}

/// Slot address inside either the open chest (`0..24`) or the player's pack (`0..30`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChestCursor {
    /// Slot index `0..24` inside the currently open chest.
    Chest(usize),
    /// Slot index `0..30` inside [`Pack::grid`].
    Pack(usize),
}

/// World-wide placed chests and torches, plus pending client actions for multiplayer sessions.
#[derive(Resource, Default, Debug)]
pub struct ChestWorld {
    /// Authoritative local or server-replicated placed props.
    pub chests: Chests,
    /// True while connected to a server that owns chest authority.
    pub remote: bool,
    /// Outgoing actions queued for the network layer to send and drain.
    pub asks: Vec<ChestAct>,
}

impl ChestWorld {
    /// Applies `act` locally when offline, or queues it in [`Self::asks`] when [`Self::remote`].
    pub fn emit(&mut self, act: ChestAct) {
        if self.remote {
            self.asks.push(act);
            return;
        }
        match act {
            ChestAct::Place {
                body,
                dir,
                yaw,
                kind,
            } => {
                let _ = self.chests.place(body, dir, yaw, kind);
            }
            ChestAct::Break { id } => {
                let _ = self.chests.remove(id);
            }
            ChestAct::SetSlot { id, slot, item } => {
                let _ = self.chests.set_slot(id, slot, item);
            }
            ChestAct::Sort { id } => {
                let _ = self.chests.sort_slots(id);
            }
        }
    }
}

/// Currently inspected chest and selected slot cursor while [`ChestScreen::Open`].
#[derive(Resource, Default, Debug)]
pub struct OpenChest {
    /// World id of the open chest prop, if any.
    pub id: Option<u64>,
    /// Selected slot awaiting a second click to swap or merge.
    pub selected: Option<ChestCursor>,
}

/// Registers placed chests, flickering torches, dual-grid storage UI, and save hooks.
pub fn plugin(app: &mut App) {
    let sweep = on_timer(Duration::from_secs_f32(SYNC_STEP));
    let ui_dirty = resource_changed::<ChestWorld>
        .or_else(resource_changed::<Pack>)
        .or_else(resource_changed::<OpenChest>);
    let prop_near = in_control
        .and_then(on_foot)
        .and_then(any_with_component::<show::PropEntity>);
    let open = in_state(ChestScreen::Open);
    app.init_state::<ChestScreen>()
        .init_resource::<ChestWorld>()
        .init_resource::<OpenChest>()
        .add_systems(OnEnter(ChestScreen::Open), ui::spawn)
        .add_systems(OnExit(ChestScreen::Open), open::clear)
        .add_observer(ui::on_activate)
        .add_systems(
            Update,
            (
                save::load.run_if(resource_added::<ActiveWorld>),
                place::place.run_if(in_control.and_then(on_foot)),
                place::break_prop.run_if(prop_near.clone()),
                open::aim_and_open.run_if(prop_near),
                open::close.run_if(open.clone()),
                ui::on_right_click.run_if(open.clone()),
                ui::paint.run_if(open.and_then(ui_dirty)),
                show::sync
                    .run_if(sweep.or_else(resource_changed::<ChestWorld>))
                    .run_if(resource_exists::<Anchor>)
                    .run_if(resource_exists::<Assets<Mesh>>)
                    .run_if(resource_exists::<Assets<StandardMaterial>>),
                show::flicker.run_if(any_with_component::<show::TorchFlame>),
                save::store.run_if(resource_changed::<ChestWorld>),
            )
                .chain(),
        );
}
