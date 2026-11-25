//! Terrain dig synchronization in a session: local craters go to the server, and server snapshots
//! and remote digs update [`Digs`] and trigger chunk remeshing without echoing back.

use abstractions_net::wire::{ClientMsg, DigOp, ServerMsg};
use bevy::prelude::*;

use super::link::{FromServer, Link, Receive, Role};
use crate::world::digs::{DigStamp, Digs, TerrainDug};

/// Tracks remote dig stamps emitted locally so they are not re-sent to the server.
#[derive(Resource, Debug, Default)]
pub struct DigSync {
    /// Stamps received from the server and written to [`TerrainDug`] for chunk remeshing.
    remote: Vec<DigStamp>,
}

/// Converts a local [`DigStamp`] into a wire [`DigOp`].
pub(crate) fn to_op(stamp: DigStamp) -> DigOp {
    DigOp {
        body: stamp.planet,
        dir: [
            stamp.dir[0] as f32,
            stamp.dir[1] as f32,
            stamp.dir[2] as f32,
        ],
        radius: stamp.radius,
        depth: stamp.depth,
    }
}

/// Converts a wire [`DigOp`] into a local [`DigStamp`].
pub(crate) fn from_op(op: DigOp) -> DigStamp {
    DigStamp {
        planet: op.body,
        dir: [
            f64::from(op.dir[0]),
            f64::from(op.dir[1]),
            f64::from(op.dir[2]),
        ],
        radius: op.radius,
        depth: op.depth,
    }
}

/// Resets dig sync state when a link opens; joined clients start from the server snapshot.
fn reset(link: Res<Link>, mut sync: ResMut<DigSync>, mut digs: ResMut<Digs>) {
    sync.remote.clear();
    if matches!(link.role, Role::Join { .. }) {
        *digs = Digs::default();
    }
}

/// Applies dig snapshots and live remote digs into [`Digs`] and emits guarded [`TerrainDug`].
fn receive(
    mut from: MessageReader<FromServer>,
    mut sync: ResMut<DigSync>,
    mut digs: ResMut<Digs>,
    mut dug: MessageWriter<TerrainDug>,
) {
    let mut push = |op: DigOp| {
        let stamp = from_op(op);
        digs.push(stamp);
        sync.remote.push(stamp);
        dug.write(TerrainDug(stamp));
    };
    for FromServer(msg) in from.read() {
        match msg {
            ServerMsg::Digs(batch) => batch.iter().copied().for_each(&mut push),
            ServerMsg::Dig(op) => push(*op),
            _ => {}
        }
    }
}

/// Sends locally dug craters to the server, skipping stamps echoed from [`receive`].
fn send(mut link: ResMut<Link>, mut sync: ResMut<DigSync>, mut dug: MessageReader<TerrainDug>) {
    for TerrainDug(stamp) in dug.read() {
        if let Some(pos) = sync.remote.iter().position(|s| s == stamp) {
            sync.remote.swap_remove(pos);
        } else {
            link.send(ClientMsg::Dig(to_op(*stamp)));
        }
    }
}

/// Registers terrain dig synchronization for networked sessions. Every system waits for [`Link`],
/// so single player runs none.
pub(super) fn plugin(app: &mut App) {
    let linked = resource_exists::<Link>;
    app.add_message::<TerrainDug>()
        .init_resource::<Digs>()
        .init_resource::<DigSync>()
        .add_systems(
            Update,
            (
                reset.run_if(resource_added::<Link>),
                receive
                    .after(Receive)
                    .run_if(linked)
                    .run_if(on_message::<FromServer>),
                send.run_if(linked).run_if(on_message::<TerrainDug>),
            )
                .chain(),
        );
}
