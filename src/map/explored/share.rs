//! Sharing the explored ground on a server: the cells uncovered here go out every two
//! seconds, and the cells of the other players come back, whole in the welcome and new ones
//! after.

use std::time::Duration;

use abstractions_net::wire::{ClientMsg, ServerMsg};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::Explored;
use crate::net::{FromServer, Link, Receive};
use crate::screens::Screen;
use crate::universe::BodyId;

/// Seconds between two sends of the cells uncovered here.
const EVERY: f32 = 2.0;

/// The ground of the last welcome, merged again once its world is in play: joining starts a
/// new world, which replaces the explored ground at the end of the frame of the welcome.
#[derive(Resource, Debug, Default)]
struct Welcomed(Vec<(BodyId, Vec<u64>)>);

/// Marks the cells the server sends, and the whole ground of a welcome.
fn take(
    mut from: MessageReader<FromServer>,
    mut explored: ResMut<Explored>,
    mut welcomed: ResMut<Welcomed>,
) {
    for FromServer(msg) in from.read() {
        match msg {
            ServerMsg::Explored { body, cells } => explored.reveal_cells(*body, cells),
            ServerMsg::Welcome(welcome) => {
                for (body, words) in &welcome.explored {
                    explored.merge_words(*body, words);
                }
                welcomed.0.clone_from(&welcome.explored);
            }
            _ => {}
        }
    }
}

/// Merges the ground of the welcome into the world the welcome started.
fn settle(mut welcomed: ResMut<Welcomed>, mut explored: ResMut<Explored>) {
    for (body, words) in welcomed.0.drain(..) {
        explored.merge_words(body, &words);
    }
}

/// Sends the cells uncovered here since the last send, one message a body.
fn send(mut link: ResMut<Link>, mut explored: ResMut<Explored>) {
    let bodies: Vec<BodyId> = explored.bodies().map(|(body, _)| body).collect();
    for body in bodies {
        let cells = explored.take_new(body);
        if !cells.is_empty() {
            link.send(ClientMsg::Explored { body, cells });
        }
    }
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
        .add_systems(OnEnter(Screen::Gameplay), settle);
}
