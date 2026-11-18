//! The jitter buffer: the last state samples of a remote body, and the smooth state between
//! them a little in the past.
//!
//! A [`Buffer`] keeps the snapshots of one robot, ship or companion, and draws it at `now`
//! minus a delay that rides over late and lost datagrams. Positions are cubic Hermite curves
//! when both ends carry a velocity and straight lines otherwise (the companion). Rotations
//! slerp along the shortest path. The rest of a body (pose, gear, glow) is an opaque `extra`
//! the caller blends itself with the weight it is handed.
//!
//! Positions are in the frame of the record. The buffer cannot tell frames apart, so it would
//! blend a surface position into an orbital one: the caller must [`Buffer::reset`] it whenever
//! the frame of a body changes.

mod buffer;
#[cfg(test)]
mod buffer_tests;
mod ring;
#[cfg(test)]
mod ring_tests;
mod snap;
mod spread;

pub use buffer::{Buffer, DELAY_MAX, DELAY_MIN, DRIFT};
pub use ring::CAPACITY;
pub use snap::{EXTRAPOLATE_MAX, Snap, View};
pub use spread::WINDOW;
