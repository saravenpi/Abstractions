//! Stable identifiers of galaxies, stars, planets and moons: enough to regenerate each from the seed.
//!
//! The types live in the network crate, which the server shares; this module keeps their path.

pub use abstractions_net::ids::{BodyId, GalaxyId, StarId};
