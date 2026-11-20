//! The robot hurt: a [`Blow`](super::blow::Blow) lands on the part nearest its contact point and
//! lowers its health in [`Condition`](crate::condition::Condition); the body then shows it.
//!
//! - Dents and scorch lie on the plates of a hurt part, more of them the worse it is ([`scar`]).
//! - A badly hurt part hangs a loose panel that swings ([`panel`]) and throws sparks ([`spark`]).
//! - A hurt head dims and flickers its screen ([`eye`]).
//! - A hurt leg limps, two lost legs crawl and a broken arm hangs ([`limp`]).
//!
//! What a hurt part can no longer do is read by the systems that drive it, from the same
//! condition: see the effects on [`Condition`](crate::condition::Condition).

pub mod eye;
pub mod limp;
mod panel;
pub mod plates;
mod scar;
mod spark;
mod take;
pub mod tune;

#[cfg(test)]
mod tests_look;
#[cfg(test)]
mod tests_struck;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::AvatarSet;
use super::animate::Skeleton;
use super::blow::Blow;
use crate::condition::Condition;
use crate::screens::playing;
use panel::Loose;
pub use scar::Scars;
use spark::Spark;

/// Run condition: the marks' meshes and materials can be made and are not yet.
fn unstocked(
    meshes: Option<Res<Assets<Mesh>>>,
    materials: Option<Res<Assets<StandardMaterial>>>,
    scars: Option<Res<Scars>>,
) -> bool {
    meshes.is_some() && materials.is_some() && scars.is_none()
}

/// Run condition: a body was just spawned, which has none of the marks of the harm it carries.
fn fresh(bodies: Query<(), Added<Skeleton>>) -> bool {
    !bodies.is_empty()
}

/// Registers the harm of blows and how the body shows it.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Condition>();
    let ready = resource_exists::<Scars>;
    let changed = resource_changed::<Condition>.or_else(fresh).and_then(ready);
    let drizzle = on_timer(Duration::from_secs_f32(tune::DRIZZLE)).and_then(ready);
    app.add_systems(
        Update,
        (
            scar::stock.run_if(unstocked),
            take::take.run_if(on_message::<Blow>).run_if(playing),
            (scar::mark, panel::hang)
                .run_if(changed)
                .in_set(AvatarSet::Animate),
            panel::sway
                .run_if(any_with_component::<Loose>)
                .in_set(AvatarSet::Animate),
            spark::burst
                .run_if(on_message::<Blow>.and_then(ready))
                .run_if(playing),
            spark::drizzle.run_if(drizzle).run_if(playing),
            spark::fly.run_if(any_with_component::<Spark>),
        ),
    );
}
