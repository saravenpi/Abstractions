//! What happens to the ship, as messages the rest of the game reads: lift-off, touchdown, the
//! warp drive, boarding, the headlight and crashes.

use bevy::prelude::*;

/// Something that happened to the ship. Read it with `MessageReader<ShipEvent>`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum ShipEvent {
    /// The ship left the ground it rested on.
    Liftoff,
    /// The ship touched the ground after flying.
    Touchdown {
        /// Speed of the touchdown in metres per second. Above 6 on the gear it damages the ship.
        speed: f64,
        /// Damage of the ship after the touchdown, from 0 to 1.
        damage: f64,
    },
    /// The warp drive did not spool or stopped spooling: the ship is landed, in an atmosphere
    /// or inside a lock sphere.
    WarpRefused,
    /// The warp drive started to spool.
    WarpSpooling,
    /// The ship went into warp.
    WarpEngaged,
    /// The ship dropped out of warp.
    WarpDropped,
    /// The player took the seat.
    Boarded,
    /// The player left the ship.
    Left,
    /// The pilot switched the headlight: on when true.
    Headlight(bool),
}

/// A crash: a touchdown at or above the crash speed of the point that took it, 25 m/s on the
/// locked gear and 10 m/s on the hull or a folded gear. Nothing damages the ship for it yet;
/// read it with `MessageReader<Impact>`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Impact {
    /// Approach speed of the point that hit, in metres per second along the ground normal.
    pub speed: f64,
    /// Where it hit, in the axes of the surface frame of the planet the ship is on.
    pub at: bevy::math::DVec3,
}
