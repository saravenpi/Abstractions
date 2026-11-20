//! The explored ground of a body as a sparse set of quadtree leaves, kept as runs.
//!
//! A leaf is a `u64` key on the cube-sphere of one body: bits 62 to 60 hold the face (0 to 5),
//! bits 59 to 54 the depth (1 to [`DEPTH_MAX`]), bits 53 to 0 the Morton code of the leaf's
//! column and row. A key stays under 2^63, so a text format can hold it. The net crate never
//! asks how big a leaf is: the game chooses the depth of each body.

mod convert;
mod merge;
mod run;
mod set;
#[cfg(test)]
mod tests;

pub use run::Run;
pub use set::Leaves;

/// Deepest level of a leaf key: 2^27 leaves along the edge of a face.
pub const DEPTH_MAX: u64 = 27;

/// Returns true when `leaf` names a leaf: a real face, a depth in range and a Morton code that
/// fits the depth.
pub fn valid(leaf: u64) -> bool {
    let (face, depth, morton) = parts(leaf);
    leaf >> 63 == 0 && face < 6 && (1..=DEPTH_MAX).contains(&depth) && morton < 1 << (2 * depth)
}

/// Returns the face, the depth and the Morton code of `leaf`.
pub fn parts(leaf: u64) -> (u64, u64, u64) {
    ((leaf >> 60) & 7, (leaf >> 54) & 63, leaf & ((1 << 54) - 1))
}

/// Returns the key of the leaf with `face`, `depth` and Morton code `morton`.
pub fn key(face: u64, depth: u64, morton: u64) -> u64 {
    (face << 60) | (depth << 54) | morton
}
