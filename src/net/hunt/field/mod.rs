//! What the others see of the field the same as this player does: the blows on robots, the wounds of
//! animals and the spears that plug them, the sound of a blow, the attacks of animals and the pieces
//! broken off rocks.
//!
//! Each act has the two halves the hooks of [`super::hooks`] describe. The sender reads the message
//! the game already writes where the thing happens and tells the server. The receiver drains
//! [`super::Echoes`] and applies what came through the same local path, so both screens run the same
//! code. What the receiver plays is noted in [`super::Hunted`], so that it is not sent back.

mod attacks;
mod codec;
mod duel;
mod hits;
mod impacts;
mod rocks;
mod tune;
mod wounds;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use super::hooks::Echoes;
use super::take;
use crate::audio::Impact;
use crate::avatar::blow::Blow;
use crate::avatar::strike::Strike;
use crate::fauna::bleed::{Bled, Wound};
use crate::net::link::Link;
use crate::net::state::Remotes;
use crate::space::{Anchor, Clock, Location};
use crate::world::Terrain;
use crate::world::rocks::{RockTaken, Taken};

/// Run condition: some echo of a hunting act waits to be played.
fn echoing(echoes: Res<Echoes>) -> bool {
    !(echoes.blows.is_empty() && echoes.impacts.is_empty() && echoes.attacks.is_empty())
        || !(echoes.rocks.is_empty() && echoes.totals.is_empty())
}

/// Run condition: a wound or a plug of the others waits for its animal.
fn bleeding(echoes: Res<Echoes>) -> bool {
    !(echoes.wounds.is_empty() && echoes.plugs.is_empty())
}

/// Run condition: this machine knows where it stands.
fn placed(anchor: Option<Res<Anchor>>, location: Option<Res<Location>>) -> bool {
    anchor.is_some() && location.is_some()
}

/// Registers the field acts of a session. Every system waits for a link and for what it reads, so
/// single player runs none.
pub fn plugin(app: &mut App) {
    app.add_message::<Impact>()
        .add_message::<Blow>()
        .add_message::<Bled>()
        .add_message::<RockTaken>()
        .add_plugins((bodies, fighting, wounded, ground));
}

/// Registers the acts on robots: blows and the sound of impacts.
fn bodies(app: &mut App) {
    let linked = resource_exists::<Link>;
    let anchor = resource_exists::<Anchor>;
    app.add_systems(
        Update,
        (
            hits::play.run_if(linked).run_if(echoing).run_if(anchor),
            duel::duel
                .run_if(linked)
                .run_if(any_with_component::<Strike>)
                .run_if(resource_exists::<Remotes>)
                .run_if(anchor),
            impacts::play.run_if(linked).run_if(echoing).run_if(placed),
            impacts::send
                .run_if(linked)
                .run_if(on_message::<Impact>)
                .run_if(placed),
        )
            .chain()
            .after(take::receive),
    );
}

/// Registers the attacks of animals.
fn fighting(app: &mut App) {
    let linked = resource_exists::<Link>;
    let here = resource_exists::<Location>;
    let drawn = resource_exists::<Terrain>.and_then(resource_exists::<Clock>);
    app.add_systems(
        Update,
        (
            attacks::play
                .run_if(linked)
                .run_if(echoing)
                .run_if(drawn)
                .run_if(here),
            attacks::send
                .run_if(linked)
                .run_if(attacks::began)
                .run_if(here),
        )
            .chain()
            .after(take::receive),
    );
}

/// Registers the wounds of animals and the spears that plug them.
fn wounded(app: &mut App) {
    let linked = resource_exists::<Link>;
    let here = resource_exists::<Location>;
    app.add_systems(
        Update,
        (
            wounds::send
                .run_if(linked)
                .run_if(on_message::<Bled>)
                .run_if(here)
                .run_if(resource_exists::<Clock>),
            wounds::play
                .run_if(linked)
                .run_if(bleeding)
                .run_if(placed)
                .run_if(resource_exists::<Clock>),
            wounds::plugged
                .run_if(linked)
                .run_if(bleeding)
                .run_if(any_with_component::<Wound>),
            wounds::plugs
                .run_if(linked)
                .run_if(any_with_component::<Wound>)
                .run_if(here),
        )
            .chain()
            .after(take::receive),
    );
}

/// Registers the acts on the ground: the pieces broken off rocks.
fn ground(app: &mut App) {
    let linked = resource_exists::<Link>;
    let taken = resource_exists::<Taken>;
    app.add_systems(
        Update,
        (
            rocks::play.run_if(linked).run_if(echoing).run_if(taken),
            rocks::send
                .run_if(linked)
                .run_if(on_message::<RockTaken>)
                .run_if(taken),
        )
            .chain()
            .after(take::receive),
    );
}
