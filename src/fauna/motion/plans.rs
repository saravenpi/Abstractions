//! The body plans that do not walk: crawling, pulsing, oozing, tumbling, flying and swimming.

pub mod air;
pub mod ground;
pub mod water;

pub use air::{fly, swarm};
pub use ground::{ooze, slither, tumble};
pub use water::{pulse, swim};
