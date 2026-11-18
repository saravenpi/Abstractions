//! Quantization: how rotations, positions and small numbers shrink to a few bytes on the wire.
//!
//! Every function is pure. A value out of range is clamped, never wrapped, and a value that is
//! not a number packs as zero.

mod position;
#[cfg(test)]
mod position_tests;
mod rotation;
#[cfg(test)]
mod rotation_tests;
mod scalar;
#[cfg(test)]
mod scalar_tests;

pub use position::{MM_RANGE, pack_mm, unpack_mm};
pub use rotation::{pack_rotation, unpack_rotation};
pub use scalar::{
    pack_i8, pack_i16, pack_u16, pack_unit, unpack_i8, unpack_i16, unpack_u16, unpack_unit,
};
