//! Throwing blood: the drops a wound sends off, with the scatter of a jet.

use bevy::prelude::*;

use super::tune::{BURST, OOZE, PARCEL, SCATTER, WET};
use super::wound::Cut;
use crate::condition::luck;
use crate::effects::Fling;
use crate::life::blood::{Blood, drop_radius};

/// A jet of blood: where it leaves, the way and how fast, and what colour it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jet {
    /// Where it leaves, in render space.
    pub at: Vec3,
    /// Unit direction it leaves along.
    pub dir: Vec3,
    /// Speed it leaves at, in metres per second.
    pub speed: f32,
    /// Linear RGB colour of the blood.
    pub colour: [f32; 3],
    /// Index of the species that bleeds.
    pub species: u16,
}

/// Returns `dir` turned a little, by up to [`SCATTER`] radians times `wide`, in a way the roll `key`
/// picks.
pub fn scatter(dir: Vec3, wide: f32, key: u32) -> Vec3 {
    let roll = |n: u32| luck(key ^ n) * 2.0 - 1.0;
    let jitter = Vec3::new(roll(0x51), roll(0xa3), roll(0x7f)) * SCATTER * wide;
    (dir + jitter).normalize_or(dir)
}

/// Throws a drop of `volume` cubic metres along `jet`, scattered by the roll `key`.
pub fn throw(flings: &mut MessageWriter<Fling>, jet: &Jet, (volume, key): (f32, u32)) {
    flings.write(Fling {
        at: jet.at,
        velocity: scatter(jet.dir, 1.0, key) * jet.speed,
        radius: drop_radius(volume),
        colour: jet.colour,
        wet: true,
        tag: Some(jet.species),
    });
}

/// Throws the burst of a cut `width` metres wide and `depth` deep: the blood it displaced, in drops of
/// at least [`PARCEL`], at most [`BURST`] of them.
pub fn burst(flings: &mut MessageWriter<Fling>, jet: &Jet, (width, depth): (f32, f32), key: u32) {
    let volume = (width * depth * WET).max(PARCEL);
    let count = ((volume / PARCEL).ceil() as usize).clamp(1, BURST);
    for n in 0..count as u32 {
        throw(
            flings,
            jet,
            (volume / count as f32, key.wrapping_add(n * 977)),
        );
    }
}

/// Returns the jet that `cut` of the animal `species` placed at `placed` lets out: along its surface
/// normal, at `fast` metres per second when it reached a major vessel and as an ooze when it did not,
/// the colour of an artery in the first case and of a vein in the second.
pub fn jet_of(
    placed: &GlobalTransform,
    cut: &Cut,
    blood: &Blood,
    (species, fast): (u16, f32),
) -> Jet {
    let dir = placed
        .affine()
        .transform_vector3(cut.normal)
        .normalize_or(Vec3::Y);
    Jet {
        at: placed.transform_point(cut.at),
        dir,
        speed: if cut.major { fast } else { OOZE },
        colour: blood.drop(cut.major),
        species,
    }
}
