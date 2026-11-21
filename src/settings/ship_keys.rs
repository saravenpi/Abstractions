//! Key bindings of the ship.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Key bindings of the ship. The keys steer it and the mouse only looks around.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(default)]
pub struct ShipKeys {
    /// Push the throttle forward. Held at full throttle it boosts the main engine.
    pub throttle_up: KeyCode,
    /// Pull the throttle back, down to idle. Held at idle with the flight assist on, it brakes.
    pub throttle_down: KeyCode,
    /// Push the nose down.
    pub pitch_down: KeyCode,
    /// Pull the nose up.
    pub pitch_up: KeyCode,
    /// Turn the nose left, in the ship and on the wingsuit.
    pub turn_left: KeyCode,
    /// Turn the nose right, in the ship and on the wingsuit.
    pub turn_right: KeyCode,
    /// Roll to the left.
    pub bank_left: KeyCode,
    /// Roll to the right.
    pub bank_right: KeyCode,
    /// Fire the vertical thrusters upward.
    pub rise: KeyCode,
    /// Fire the vertical thrusters downward.
    pub sink: KeyCode,
    /// Switch the flight assist on or off.
    pub assist: KeyCode,
    /// Set the throttle to zero.
    pub cut: KeyCode,
    /// Spool the warp drive, or drop out of warp.
    pub warp: KeyCode,
    /// Board the ship, or leave it.
    pub board: KeyCode,
    /// Switch the headlight in the nose on or off.
    pub headlight: KeyCode,
}

impl Default for ShipKeys {
    fn default() -> Self {
        Self {
            throttle_up: KeyCode::ShiftLeft,
            throttle_down: KeyCode::ControlLeft,
            pitch_down: KeyCode::KeyW,
            pitch_up: KeyCode::KeyS,
            turn_left: KeyCode::KeyQ,
            turn_right: KeyCode::KeyE,
            bank_left: KeyCode::KeyA,
            bank_right: KeyCode::KeyD,
            rise: KeyCode::Space,
            sink: KeyCode::KeyC,
            assist: KeyCode::KeyZ,
            cut: KeyCode::KeyX,
            warp: KeyCode::KeyJ,
            board: KeyCode::KeyF,
            headlight: KeyCode::KeyL,
        }
    }
}
