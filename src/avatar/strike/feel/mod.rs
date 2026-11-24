//! How a strike feels and reads: it goes where the crosshair points ([`sight`]) with the whole body
//! behind it ([`bend`], [`body`]), each contact throws flecks by what it struck ([`spark`]) and marks
//! the crosshair ([`mark`]), the robot strikes a rock it breaks a piece off ([`knap`]), and the tool
//! is slung over the back before the wingsuit opens or the robot boards ([`stow`]).

pub mod bend;
pub mod body;
pub mod knap;
pub mod mark;
pub mod sight;
pub mod spark;
pub mod stow;
pub mod tune;

#[cfg(test)]
mod tests_aim;
#[cfg(test)]
mod tests_body;
#[cfg(test)]
mod tests_feedback;
#[cfg(test)]
mod tests_knap;
#[cfg(test)]
mod tests_stow;

use bevy::prelude::*;

use super::super::AvatarSet;
use super::held::Held;
use super::hit::Hit;
use super::input::ask;
use super::layer::lay;
use crate::audio::Impact;
use crate::craft::pack::{Forage, Foraged, Knock, Knocked};
use crate::effects::Fling;
use crate::player::{PlayerSet, Squared};
use crate::screens::{in_control, playing};

/// Registers the feel of strikes: the stow and the facing before the hands are read, the knap over
/// the strike layer, and the marks and flecks of blows.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<sight::Sight>()
        .init_resource::<mark::Mark>()
        .init_resource::<Squared>()
        .add_message::<Impact>()
        .add_message::<Fling>()
        .add_message::<Knock>()
        .add_message::<Knocked>()
        .add_message::<Forage>()
        .add_message::<Foraged>()
        .add_systems(
            Update,
            (
                stow::stow
                    .run_if(in_control.and_then(any_with_component::<Held>))
                    .before(ask),
                stow::square.before(PlayerSet::Move).run_if(stow::squaring),
                (
                    knap::start.run_if(on_message::<Knock>),
                    knap::start_forage.run_if(on_message::<Forage>),
                    knap::knap.run_if(any_with_component::<knap::Knap>),
                )
                    .chain()
                    .after(lay)
                    .after(AvatarSet::Fill)
                    .before(AvatarSet::Animate),
                mark::note.run_if(on_message::<Hit>),
                mark::age.run_if(mark::marking),
                spark::scatter.run_if(on_message::<Impact>),
            )
                .run_if(playing),
        );
}
