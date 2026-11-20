//! Sharing the explored ground on a server: the leaves uncovered here go out every two
//! seconds, and the leaves of the other players come back, whole in the welcome and new ones
//! after.

use std::time::Duration;

use abstractions_net::leaves::Run;
use abstractions_net::wire::{ClientMsg, ServerMsg};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::Explored;
use crate::net::{FromServer, Link, Receive};
use crate::screens::Screen;
use crate::universe::BodyId;

/// Seconds between two sends of the cells uncovered here.
const EVERY: f32 = 2.0;

/// Most runs one message carries: at most 12 bytes a run, 36 KB, well under the
/// [`abstractions_net::wire::CLIENT_MESSAGE_MAX`] a server reads from a player.
pub(super) const RUNS_MAX: usize = 3000;

/// The ground of the last welcome, merged again once its world is in play: joining starts a
/// new world, which replaces the explored ground at the end of the frame of the welcome.
#[derive(Resource, Debug, Default)]
struct Welcomed(Vec<(BodyId, Vec<Run>)>);

/// Marks the leaves the server sends, and the whole ground of a welcome.
fn take(
    mut from: MessageReader<FromServer>,
    mut explored: ResMut<Explored>,
    mut welcomed: ResMut<Welcomed>,
) {
    for FromServer(msg) in from.read() {
        match msg {
            ServerMsg::Explored { body, runs } => explored.merge_runs(*body, runs),
            ServerMsg::Welcome(welcome) => {
                for (body, runs) in &welcome.explored {
                    explored.merge_runs(*body, runs);
                }
                welcomed.0.clone_from(&welcome.explored);
            }
            _ => {}
        }
    }
}

/// Merges the ground of the welcome into the world the welcome started.
fn settle(mut welcomed: ResMut<Welcomed>, mut explored: ResMut<Explored>) {
    for (body, runs) in welcomed.0.drain(..) {
        explored.merge_runs(body, &runs);
    }
}

/// Returns the messages that carry the leaves uncovered here since the last call, at most
/// [`RUNS_MAX`] runs each.
pub(super) fn messages(explored: &mut Explored) -> Vec<ClientMsg> {
    let bodies: Vec<BodyId> = explored.bodies().map(|(body, _)| body).collect();
    let mut out = Vec::new();
    for body in bodies {
        let chunks = explored.take_new(body).chunks(RUNS_MAX);
        out.extend(
            chunks
                .into_iter()
                .map(|runs| ClientMsg::Explored { body, runs }),
        );
    }
    out
}

/// Sends the leaves uncovered here since the last send.
fn send(mut link: ResMut<Link>, mut explored: ResMut<Explored>) {
    for msg in messages(&mut explored) {
        link.send(msg);
    }
}

/// Forgets the leaves uncovered with no server to send them to.
fn drop_unsent(mut explored: ResMut<Explored>) {
    explored.bypass_change_detection().drop_unsent();
}

/// Registers the sharing of the explored ground.
pub fn plugin(app: &mut App) {
    app.init_resource::<Welcomed>()
        .add_systems(
            Update,
            (
                take.after(Receive),
                send.run_if(on_timer(Duration::from_secs_f32(EVERY))),
            )
                .run_if(resource_exists::<Link>),
        )
        .add_systems(Update, drop_unsent.run_if(not(resource_exists::<Link>)))
        .add_systems(OnEnter(Screen::Gameplay), settle);
}

#[cfg(test)]
#[path = "share_tests.rs"]
mod tests;
