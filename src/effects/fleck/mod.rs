//! Flecks: drops of blood, chips and dust thrown by a blow. Each is a real body that flies under the
//! gravity of the planet and the drag of its air, and a drop that lands says where, so the owner of
//! the fleck can lay a mark there.
//!
//! The pool is fixed: [`SLOTS`] entities made once, sharing one mesh, with one material per colour.
//! A new fleck takes a free slot, or the oldest live one, so the pool never grows and nothing
//! allocates per frame.

mod fly;
mod pool;
mod tune;

#[cfg(test)]
mod tests_fleck;

use bevy::prelude::*;

pub use pool::SLOTS;

use crate::screens::{Screen, playing};

/// A fleck to throw. Send it and the pool takes a slot for it.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Fling {
    /// Where it starts, in render space.
    pub at: Vec3,
    /// Its velocity, in metres per second in render space.
    pub velocity: Vec3,
    /// Radius of the sphere, in metres.
    pub radius: f32,
    /// Linear RGB colour.
    pub colour: [f32; 3],
    /// True for a wet fleck, which shines.
    pub wet: bool,
    /// Whatever the thrower wants back when it lands: the species of a drop of blood.
    pub tag: Option<u16>,
}

/// A fleck that landed on the ground, for the one who threw it to read.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Landed {
    /// Where it landed, in render space.
    pub at: Vec3,
    /// Radius of the sphere it was, in metres.
    pub radius: f32,
    /// What the thrower tagged it with.
    pub tag: Option<u16>,
    /// Unit direction it was flying in when it landed.
    pub heading: Vec3,
}

/// Registers the fleck pool and its systems.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<Fling>()
        .add_message::<Landed>()
        .init_resource::<pool::Flecks>()
        .add_systems(OnEnter(Screen::Gameplay), pool::spawn)
        .add_systems(
            Update,
            (
                fly::fling.run_if(on_message::<Fling>),
                fly::fly.run_if(pool::flying),
            )
                .chain()
                .run_if(playing),
        );
}
