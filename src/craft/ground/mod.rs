//! The ground: tools and heaps of material the robot puts down, real objects that lie where they
//! were put until someone takes them into a pack.
//!
//! The pack screen drops a thing into [`Shed`], and the game puts it down ahead of the robot's
//! feet. The interact key takes the thing under the eyes back into the pack, as much as the pack
//! has room for. [`Ground::pile`] holds what lies, under the rule of [`abstractions_net::ground`]:
//! alone, the game is its authority and the save keeps it; in a session the server is, and
//! [`Ground::remote`] turns every drop and take into an ask that the network sends and answers.

mod ask;
mod codec;
mod hand;
mod lay;
mod save;
mod show;
mod take;
mod tune;

#[cfg(test)]
mod tests_full;
#[cfg(test)]
mod tests_round;
#[cfg(test)]
mod tests_spread;

pub use ask::{NO_ROOM, TAKEN};
pub use codec::{lying, thing};
pub use hand::{restore, shed_item, shed_slot, shed_stack};
pub use show::Lies;

use std::time::Duration;

use abstractions_net::ground::Pile;
use abstractions_net::wire::Act;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::craft::gun::Gun;
use crate::craft::pack::{Item, Stack};
use crate::saves::ActiveWorld;
use crate::screens::in_control;
use crate::space::{Anchor, on_foot};

/// One thing that can lie on the ground.
#[derive(Debug, Clone, PartialEq)]
pub enum Thing {
    /// A made tool, its wear kept in its recipe.
    Tool(Item),
    /// A crafted firearm.
    Gun(Gun),
    /// A heap of one material.
    Heap(Stack),
}

/// What lies on the ground of the world, and what this game asked a server about it.
#[derive(Resource, Debug, Default)]
pub struct Ground {
    /// The things that lie.
    pub pile: Pile,
    /// True while a server keeps the ground: drops and takes are asked of it.
    pub remote: bool,
    /// What to ask the server, oldest first. The network sends and clears it.
    pub asks: Vec<Act>,
    /// The drops asked and not answered yet, by their tag.
    pub pending: Vec<(u32, Thing)>,
    /// The thing asked for and not answered yet.
    pub asked: Option<u64>,
    /// The tag of the last drop asked.
    pub tag: u32,
}

/// The things the pack let go of this frame, to be put down ahead of the robot.
#[derive(Resource, Debug, Default)]
pub struct Shed(pub Vec<Thing>);

/// A line about the ground for the banner: the pack is full, a thing was already taken.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct Note(pub String);

/// Run condition: the pack let go of something.
fn shedding(shed: Res<Shed>) -> bool {
    !shed.0.is_empty()
}

/// Registers the ground: putting down, taking, drawing and keeping it in the save.
pub(super) fn plugin(app: &mut App) {
    let look = on_timer(Duration::from_secs_f32(tune::LOOK));
    app.init_resource::<Ground>()
        .init_resource::<Shed>()
        .add_message::<Note>()
        .add_systems(
            Update,
            (
                save::load.run_if(resource_added::<ActiveWorld>),
                lay::lay.run_if(shedding),
                take::take.run_if(
                    in_control
                        .and_then(on_foot)
                        .and_then(any_with_component::<Lies>),
                ),
                show::show
                    .run_if(look.or_else(resource_changed::<Ground>))
                    .run_if(resource_exists::<Anchor>),
                save::store.run_if(resource_changed::<Ground>),
            )
                .chain(),
        );
}
