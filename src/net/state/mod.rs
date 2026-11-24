//! The state of every player in a session: this machine sends where its robot, ship and
//! companion are, and draws those of the others.
//!
//! Sending reads the local robot, ship and companion once the frame is drawn, puts each in
//! the wire frame it belongs in (the surface of a body on foot or landed, around the body a
//! ship flies at, the frame of a star in deep space) and sends what is due in one batch.
//! Drawing keeps a jitter buffer per player and kind, samples it at universe time, moves the
//! sample from its wire frame into the frame of the render anchor and writes the puppet.

mod bodies;
mod draw;
mod fleet;
mod fleet_sync;
mod forget;
mod frame;
mod held;
mod rate;
mod rebase;
mod robot;
mod send;
mod store;
mod swing;
#[cfg(test)]
mod swing_tests;
mod tag;
mod track;
mod whereabouts;

#[cfg(test)]
mod tests;

pub(super) use bodies::Bodies;
pub(super) use frame::{Place, Pose};
pub(crate) use track::Remotes;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::link::{Link, Receive};
use crate::avatar::{self, AvatarSet};
use crate::ship::{self, ShipSet};

/// Marks a puppet: the robot, ship or companion of another player, drawn from the network.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Remote;

/// Registers what runs each frame of a session: the records that arrived, the players who
/// went, then the puppets and their names once the anchor of the frame is set.
fn frame(app: &mut App) {
    let drawn = (
        draw::robots.run_if(resource_exists::<avatar::Looks>),
        draw::ships.run_if(resource_exists::<ship::Looks>),
        draw::companions.run_if(resource_exists::<Assets<Image>>),
        draw::wounds.run_if(track::kept),
        draw::swings.run_if(track::kept),
        tag::tags,
    );
    let placed = drawn
        .chain()
        .after(ShipSet::Place)
        .before(ShipSet::Report)
        .before(AvatarSet::Animate);
    let heard = (track::ingest, fleet::hear, forget::forget)
        .chain()
        .after(Receive)
        .before(ShipSet::Input);
    app.add_systems(
        Update,
        (heard, fleet::act.after(ShipSet::Input), placed)
            .chain()
            .distributive_run_if(resource_exists::<Link>),
    );
}

/// Registers the state of the players: what arrives, what is drawn and what is sent. Every
/// system waits for a link, so single player runs none but the one that forgets puppets. Calm
/// water mirrors the puppets.
pub fn plugin(app: &mut App) {
    app.register_required_components::<Remote, crate::world::Reflected>();
    app.init_resource::<track::Remotes>()
        .init_resource::<send::Outbox>()
        .init_resource::<bodies::Bodies>()
        .add_systems(
            Update,
            (forget::clear.run_if(track::kept), fleet::reset).run_if(not(resource_exists::<Link>)),
        )
        .add_systems(
            PostUpdate,
            (
                send::send.run_if(resource_exists::<Link>),
                draw::glows.run_if(resource_exists::<Link>.and_then(draw::burning)),
            )
                .after(TransformSystems::Propagate),
        );
    frame(app);
}
