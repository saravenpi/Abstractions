//! Universe time on a player's machine: the offset to the server's clock from pings, and the slew
//! that closes it without a jump.
//!
//! Nothing here reads a clock. The caller passes its local monotonic time in seconds and the
//! universe time it shows, so every rule can be tested with made-up times.

mod offsets;
#[cfg(test)]
mod offsets_tests;
mod sample;
mod slew;
#[cfg(test)]
mod slew_tests;

pub use offsets::{Offsets, RTT_MAX, SAMPLES};
pub use sample::Sample;
pub use slew::{Clock, SLEW, SNAP};
