//! The flinch of a hit part, drawn over the walk: a part that was hit is pushed along the blow and
//! springs back. The telegraph of an attack is not drawn here: the brace bends the joints of the part
//! that strikes (see `crate::fauna::motion::brace`), so no part slides out of its socket.

use bevy::prelude::*;

use super::tune::FLINCH;
use crate::fauna::beast::{Beast, Part};

/// A part that was hit: pushed by `push` in the frame of the animal, springing back over [`FLINCH`]
/// seconds.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Flinch {
    /// Index of the part hit.
    pub part: u8,
    /// Where the blow pushed it at first, in metres.
    pub push: Vec3,
    /// Seconds since the blow.
    pub age: f32,
}

/// The drawn parts, the ones a flinch moves.
type Limbs<'w, 's> = Query<'w, 's, &'static mut Transform, With<Part>>;

/// Moves the part hit of every animal that flinches from a blow, and lets it go once it sprang back.
pub fn gesture(
    mut commands: Commands,
    time: Res<Time>,
    mut beasts: Query<(Entity, &Beast, &mut Flinch)>,
    mut parts: Limbs,
) {
    for (entity, beast, mut flinch) in &mut beasts {
        flinch.age += time.delta_secs();
        let push = flinch.push * (1.0 - flinch.age / FLINCH).max(0.0);
        let part = beast.parts.get(usize::from(flinch.part)).copied().flatten();
        if let Some(mut place) = part.and_then(|p| parts.get_mut(p).ok()) {
            place.translation += push;
        }
        if flinch.age >= FLINCH {
            commands.entity(entity).remove::<Flinch>();
        }
    }
}
