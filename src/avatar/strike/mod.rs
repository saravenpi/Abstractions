//! Strike: the robot's hands on a held tool. Directional swings, thrusts, guards and parries, and a
//! thrown spear, all timed and powered by the physics of the build in [`crate::craft::build`].
//!
//! A swing is an arc of the tool pivoted at the grip, its plane chosen by the mouse over the last
//! 120 ms, swept by the chest over the hips while two-bone IK keeps the hands on the grip: a
//! [`Layer`] laid over the blended pose of the body. The wind-up is 2·arc / ω stretched by torque
//! over the grip budget, the swing turns at ω, then the hands recover. After the transforms are
//! propagated the head is swept against the bounds of the parts of the animals and the first one
//! it meets takes a [`Hit`]; a [`Hitstop`] freezes both for a few frames and shakes the animal.
//! The strike aims where the crosshair points, with the whole body behind it, and every contact
//! reads on screen, as the `feel` module lays out.

mod act;
mod canvas;
mod dir;
mod feel;
mod flight;
mod held;
mod hit;
mod hurt;
mod ik;
mod input;
mod key;
mod landing;
mod layer;
mod outcome;
mod path;
mod pull;
mod recoil;
mod ribbon;
mod sense;
mod show;
#[cfg(feature = "dev")]
mod starter;
#[cfg(any(test, feature = "dev"))]
mod stock;
mod stop;
mod stray;
mod swish;
mod sync;
mod throw;
mod trail;
pub mod tune;
mod wear;

#[cfg(test)]
mod stage;
#[cfg(test)]
mod tests_act;
#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_flight;
#[cfg(test)]
mod tests_ground;
#[cfg(test)]
mod tests_hit;
#[cfg(test)]
mod tests_hurt;
#[cfg(test)]
mod tests_ik;
#[cfg(test)]
mod tests_outcome;
#[cfg(test)]
mod tests_stab;
#[cfg(test)]
mod tests_stock;
#[cfg(test)]
mod tests_stray;
#[cfg(test)]
mod tests_throw;
#[cfg(test)]
mod tests_trail;

pub use act::{Act, Strike};
pub use dir::Dir;
pub use feel::knap::Knap;
pub use feel::mark::Mark;
pub use feel::sight::Sight;
pub use flight::{Rest, Spear};
pub use held::{Held, Style};
pub use hit::{Hit, Tip};
pub use hurt::Hurt;
pub use layer::Layer;
pub use outcome::Outcome;
pub use sense::landed;
pub use show::{Kit, blocks, kit, tool};
pub use stop::Hitstop;
pub use swish::Missed;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::AvatarSet;
use crate::screens::{in_control, playing};
use crate::view::ViewSet;
use show::Wielded;

/// A body with a tool in its hands or on show.
type Drawn = Or<(With<Held>, With<Wielded>)>;

/// Run condition: some body holds a tool or still has one drawn.
fn armed(held: Query<(), Drawn>) -> bool {
    !held.is_empty()
}

/// Run condition: some body holds a tool and has no ribbon yet.
fn unrigged(bodies: Query<(), (With<Held>, Without<ribbon::Trailing>)>) -> bool {
    !bodies.is_empty()
}

/// Adds the systems of the frame: the hands read and posed before the body is animated, the drawn
/// tool placed after, the spears thrown and flown, then the hitstop.
fn update(app: &mut App) {
    let (hands, spears) = (hands(), spears());
    app.add_systems(
        Update,
        (
            hands
                .run_if(armed)
                .after(AvatarSet::Fill)
                .before(AvatarSet::Animate)
                .before(crate::craft::wield::choose),
            show::place
                .after(AvatarSet::Animate)
                .run_if(any_with_component::<Wielded>),
            spears.after(input::ask),
            sync::sync
                .run_if(sync::stale)
                .after(crate::craft::wield::choose)
                .after(crate::craft::wield::hold),
            wear::worn.run_if(on_message::<Hit>),
            stop::start.run_if(on_message::<Hit>),
            stop::hold
                .after(stop::start)
                .after(ViewSet::Rig)
                .run_if(stop::active),
        )
            .run_if(playing),
    );
}

/// Returns the systems of the hands: read, posed and drawn.
fn hands() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    (
        hurt::feel,
        feel::sight::look.run_if(in_control),
        input::ask.run_if(in_control),
        layer::lay,
        show::wield.run_if(armed),
    )
        .chain()
}

/// Returns the systems of the spears: thrown, then flown and picked up.
fn spears() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    (
        throw::loose.run_if(on_message::<input::Loose>),
        (throw::soar, throw::pick).run_if(any_with_component::<Spear>),
        pull::pull
            .run_if(any_with_component::<Spear>)
            .run_if(crate::screens::in_control.and_then(crate::space::on_foot)),
        (stray::note, stray::ground)
            .chain()
            .run_if(stray::astray)
            .run_if(resource_exists::<crate::space::Anchor>)
            .run_if(resource_exists::<crate::world::Terrain>),
    )
        .chain()
}

/// Returns the systems that show a swing: the whoosh and the ribbon, once the drawn tool is placed.
fn swings() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    (
        swish::swish.run_if(armed),
        ribbon::rig.run_if(unrigged),
        ribbon::trace.run_if(any_with_component::<ribbon::Trailing>),
    )
        .after(show::place)
        .run_if(playing)
}

/// Registers the strikes: the frame systems, and the blows sensed once the transforms are
/// propagated.
pub(super) fn plugin(app: &mut App) {
    crate::player::interacting(app);
    app.add_message::<Hit>()
        .add_message::<Missed>()
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
    feel::plugin(app);
    app.add_systems(Update, swings());
    #[cfg(feature = "dev")]
    app.add_systems(Update, starter::give.run_if(in_control).before(input::ask));
}
