//! Strike: the robot's hands on a held tool. Directional swings, thrusts, guards and parries, and a
//! thrown spear, all timed and powered by the physics of the build in [`crate::craft::build`].
//!
//! A swing is an arc of the tool pivoted at the grip, its plane chosen by the mouse over the last
//! 120 ms, swept by the chest over the hips while two-bone IK keeps the hands on the grip: a
//! [`Layer`] laid over the blended pose of the body. The wind-up is 2·arc / ω stretched by torque
//! over the grip budget, the swing turns at ω, then the hands recover. After the transforms are
//! propagated the head is swept against the bounds of the parts of the animals and the first one
//! it meets takes a [`Hit`]; a [`Hitstop`] freezes both for a few frames and shakes the animal.

mod act;
mod dir;
mod flight;
mod held;
mod hit;
mod ik;
mod input;
mod key;
mod layer;
mod path;
mod sense;
mod show;
#[cfg(feature = "dev")]
mod starter;
#[cfg(any(test, feature = "dev"))]
mod stock;
mod stop;
mod throw;
pub mod tune;

#[cfg(test)]
mod stage;
#[cfg(test)]
mod tests_act;
#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_flight;
#[cfg(test)]
mod tests_hit;
#[cfg(test)]
mod tests_ik;
#[cfg(test)]
mod tests_stock;

pub use act::{Act, Strike};
pub use dir::Dir;
pub use flight::{Rest, Spear};
pub use held::{Held, Style};
pub use hit::Hit;
pub use layer::Layer;
pub use stop::Hitstop;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::AvatarSet;
use crate::screens::{in_control, playing};
use crate::view::ViewSet;
use show::Wielded;

/// Run condition: some body holds a tool or still has one drawn.
fn armed(held: Query<(), Or<(With<Held>, With<Wielded>)>>) -> bool {
    !held.is_empty()
}

/// Adds the systems of the frame: the hands read and posed before the body is animated, the drawn
/// tool placed after, the spears thrown and flown, then the hitstop.
fn update(app: &mut App) {
    let hands = (
        input::ask.run_if(in_control),
        layer::lay,
        show::wield.run_if(armed),
    )
        .chain();
    let spears = (
        throw::loose.run_if(on_message::<input::Loose>),
        (throw::soar, throw::pick).run_if(any_with_component::<Spear>),
    )
        .chain();
    app.add_systems(
        Update,
        (
            hands
                .run_if(armed)
                .after(AvatarSet::Fill)
                .before(AvatarSet::Animate),
            show::place
                .after(AvatarSet::Animate)
                .run_if(any_with_component::<Wielded>),
            spears.after(input::ask),
            stop::start.run_if(on_message::<Hit>),
            stop::hold
                .after(stop::start)
                .after(ViewSet::Rig)
                .run_if(stop::active),
        )
            .run_if(playing),
    );
}

/// Registers the strikes: the frame systems, and the blows sensed once the transforms are
/// propagated.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<Hit>()
        .add_message::<input::Loose>()
        .init_resource::<dir::Strokes>()
        .init_resource::<Hitstop>()
        .add_systems(
            PostUpdate,
            (
                sense::sense.run_if(sense::swinging),
                throw::pierce.run_if(any_with_component::<Spear>),
            )
                .after(TransformSystems::Propagate)
                .run_if(playing),
        );
    update(app);
    #[cfg(feature = "dev")]
    app.add_systems(Update, starter::give.run_if(in_control).before(input::ask));
}
