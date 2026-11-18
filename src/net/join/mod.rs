//! Joining a server and leaving it: the hello, the welcome that starts its world, the players
//! who come and go, the shared log, and the end of the session with a reason in plain words.

mod dial;
pub(in crate::net) mod end;
mod leave;
mod roster;
mod shared;
pub(in crate::net) mod world;

pub use dial::{join, opts};
pub use end::end;
pub use leave::{keep_record, leave, quit};
pub use roster::{notify, pending, roster, welcome};
pub use shared::{share, take_changes};
pub use world::{merge, record_of, world_of};

use std::time::Duration;

use bevy::prelude::*;

use super::clock::{NetStats, SessionClock};
use super::link::{FreshKey, Inbox, Link};

/// Time between two records of the player's place and ship sent to the server.
pub const RECORD_EVERY: Duration = Duration::from_secs(30);

/// Inserts `link` and the resources that live as long as it.
pub fn open(commands: &mut Commands, link: Link) {
    commands.insert_resource(link);
    commands.insert_resource(Inbox::default());
    commands.insert_resource(FreshKey::default());
    commands.insert_resource(SessionClock::default());
    commands.insert_resource(NetStats::default());
}

/// Removes the link and the resources that live as long as it.
pub fn shut(commands: &mut Commands) {
    commands.remove_resource::<Link>();
    commands.remove_resource::<Inbox>();
    commands.remove_resource::<FreshKey>();
    commands.remove_resource::<SessionClock>();
    commands.remove_resource::<NetStats>();
}
