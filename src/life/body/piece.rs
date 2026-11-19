//! Building single parts: a plain part of a kind and shape, then placed, turned and given a tissue.

use bevy::math::{Quat, Vec3};

use super::{Part, PartKind, ShapeKind};

/// Returns a part of `kind` and `shape` with extent `size`, at the origin of no parent, in the main tissue.
pub fn piece(kind: PartKind, shape: ShapeKind, size: Vec3) -> Part {
    Part {
        kind,
        parent: None,
        socket: Vec3::ZERO,
        rest: Quat::IDENTITY,
        shape,
        size,
        taper: 1.0,
        tissue: 0,
        side: 0,
    }
}

impl Part {
    /// Returns this part with its socket at `socket` in the parent frame.
    pub fn at(self, socket: Vec3) -> Self {
        Self { socket, ..self }
    }

    /// Returns this part with the rest rotation `rest` in the parent frame.
    pub fn turned(self, rest: Quat) -> Self {
        Self { rest, ..self }
    }

    /// Returns this part made of tissue `tissue`, with tip size over base size `taper`.
    pub fn of(self, tissue: u8, taper: f32) -> Self {
        Self {
            tissue,
            taper,
            ..self
        }
    }
}
