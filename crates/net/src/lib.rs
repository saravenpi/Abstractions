//! The multiplayer of Abstractions without the game: what a player and a server say to each
//! other, and the server's logic.
//!
//! The game and the self-hosted server both build on this crate, so they cannot disagree on a
//! byte. It does not depend on Bevy. The `ecs` feature only adds the derives the game needs to
//! use [`log::PlayerId`] as a component and [`log::Changes`] as a resource.
//!
//! - [`ids`] names galaxies, stars, planets and moons.
//! - [`log`] is the list of what players changed in a world.
//! - [`leaves`] holds the explored ground of a body as runs of quadtree leaves.
//! - [`pack`] shrinks rotations, positions and small numbers to a few bytes.
//! - [`wire`] is the protocol: state records, messages and how they sit on a stream.
//! - [`clock`] and [`jitter`] keep players in time and remote bodies smooth.
//! - [`host`] is the seam between the network and the server's logic.
//! - [`room`] is the server's logic and [`store`] its world file.
//! - [`io`], [`trust`] and [`lan`] are the network itself: QUIC, certificates and the LAN beacon.

pub mod clock;
pub mod host;
pub mod ids;
pub mod io;
pub mod jitter;
pub mod lan;
pub mod leaves;
pub mod log;
pub mod pack;
pub mod room;
pub mod store;
pub mod trust;
pub mod wire;
