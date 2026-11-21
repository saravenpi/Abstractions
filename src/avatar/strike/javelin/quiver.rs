//! Quiver: the shaft of a javelin that just planted itself shakes about its buried tip, the shake
//! growing with the speed it arrived at and dying away within a second.

use bevy::prelude::*;

use super::tune::{QUIVER, QUIVER_LIFE};

/// The wobble of a planted javelin, on the javelin until it stands still.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Quiver {
    /// The place of the shaft at rest.
    pub rest: Quat,
    /// The unit axis the shaft rocks about, in render axes.
    pub about: Vec3,
    /// The first swing, in radians.
    pub swing: f32,
    /// Seconds since it planted.
    pub age: f32,
}

impl Quiver {
    /// Returns the wobble of a shaft planted along the unit `axis` at `rest`, arriving at
    /// `velocity`: it rocks across the way it came in.
    pub fn of(rest: Quat, axis: Vec3, velocity: Vec3) -> Self {
        let [gain, most, ..] = QUIVER;
        Self {
            rest,
            about: axis
                .cross(velocity)
                .normalize_or(axis.any_orthonormal_vector()),
            swing: (gain * velocity.length()).min(most),
            age: 0.0,
        }
    }
}

/// Returns the turn of the shaft `age` seconds after it planted with a first swing of `swing`
/// radians: a sine at the frequency of [`QUIVER`] that falls by e every decay time.
pub fn wobble(swing: f32, age: f32) -> f32 {
    let [_, _, hertz, decay] = QUIVER;
    swing * (-age / decay).exp() * (std::f32::consts::TAU * hertz * age).sin()
}

/// Rocks every planted javelin about its buried tip, and stands it still once the wobble is over.
pub fn quiver(
    mut commands: Commands,
    time: Res<Time>,
    mut shafts: Query<(Entity, &mut Quiver, &mut Transform)>,
) {
    for (entity, mut quiver, mut place) in &mut shafts {
        quiver.age += time.delta_secs();
        if quiver.age >= QUIVER_LIFE {
            place.rotation = quiver.rest;
            commands.entity(entity).remove::<Quiver>();
            continue;
        }
        let turn = Quat::from_axis_angle(quiver.about, wobble(quiver.swing, quiver.age));
        place.rotation = turn * quiver.rest;
    }
}
