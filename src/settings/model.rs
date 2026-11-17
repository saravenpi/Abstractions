//! The persisted settings data model.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Every user setting, saved to `settings.toml` in the config directory.
#[derive(Resource, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Settings {
    /// Rendering options.
    pub graphics: Graphics,
    /// Mouse and keyboard options.
    pub controls: Controls,
    /// Volume options.
    pub audio: Audio,
    /// World generation options.
    pub world: WorldPrefs,
}

/// Rendering options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Graphics {
    /// Size of one screen pixel block, in window pixels (1 to 8).
    pub pixel_size: u32,
    /// Chunk radius streamed around the player (2 to 8).
    pub render_distance: u32,
    /// Whether the sun casts shadows.
    pub shadows: bool,
    /// Whether presentation waits for vertical sync.
    pub vsync: bool,
    /// Whether the window is borderless fullscreen.
    pub fullscreen: bool,
    /// Whether screen-space ambient occlusion darkens creases.
    pub ambient_occlusion: bool,
    /// Whether bright areas glow.
    pub bloom: bool,
    /// Whether clouds are drawn in the sky.
    pub clouds: bool,
    /// Color levels per channel kept by the retro banding (8 to 64).
    pub color_levels: u32,
    /// Vertical field of view in degrees (60 to 110).
    pub fov: u32,
    /// Whether the frame rate is shown in a corner.
    pub show_fps: bool,
    /// Whether Solari ray traced lighting is requested; it takes effect at once when the GPU supports it.
    pub ray_tracing: bool,
}

impl Default for Graphics {
    fn default() -> Self {
        Self {
            pixel_size: 3,
            render_distance: 4,
            shadows: true,
            vsync: true,
            fullscreen: false,
            ambient_occlusion: true,
            bloom: true,
            clouds: true,
            color_levels: 24,
            fov: 75,
            show_fps: false,
            ray_tracing: false,
        }
    }
}

/// Mouse and keyboard options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Controls {
    /// Mouse look multiplier (0.1 to 3.0).
    pub mouse_sensitivity: f32,
    /// Whether moving the mouse up looks down.
    pub invert_y: bool,
    /// Key bindings.
    pub keys: Keymap,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            invert_y: false,
            keys: Keymap::default(),
        }
    }
}

/// Key bindings for every player action.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(default)]
pub struct Keymap {
    /// Walk forward.
    pub forward: KeyCode,
    /// Walk backward.
    pub back: KeyCode,
    /// Strafe left.
    pub left: KeyCode,
    /// Strafe right.
    pub right: KeyCode,
    /// Jump, or swim up in water.
    pub jump: KeyCode,
    /// Run faster.
    pub sprint: KeyCode,
    /// Ship controls, used while seated in the ship and to board it.
    pub ship: ShipKeys,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            back: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            jump: KeyCode::Space,
            sprint: KeyCode::ShiftLeft,
            ship: ShipKeys::default(),
        }
    }
}

/// Key bindings of the ship.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(default)]
pub struct ShipKeys {
    /// Push the throttle forward.
    pub thrust: KeyCode,
    /// Pull the throttle back, down to reverse.
    pub brake: KeyCode,
    /// Turn the nose left.
    pub yaw_left: KeyCode,
    /// Turn the nose right.
    pub yaw_right: KeyCode,
    /// Roll to the left.
    pub roll_left: KeyCode,
    /// Roll to the right.
    pub roll_right: KeyCode,
    /// Fire the vertical thrusters upward.
    pub up: KeyCode,
    /// Fire the vertical thrusters downward.
    pub down: KeyCode,
    /// Boost the main engine.
    pub boost: KeyCode,
    /// Switch the flight assist on or off.
    pub assist: KeyCode,
    /// Set the throttle to zero.
    pub cut: KeyCode,
    /// Spool the warp drive, or drop out of warp.
    pub warp: KeyCode,
    /// Switch between the cockpit and the chase view.
    pub view: KeyCode,
    /// Board the ship, or leave it.
    pub board: KeyCode,
}

impl Default for ShipKeys {
    fn default() -> Self {
        Self {
            thrust: KeyCode::KeyW,
            brake: KeyCode::KeyS,
            yaw_left: KeyCode::KeyA,
            yaw_right: KeyCode::KeyD,
            roll_left: KeyCode::KeyQ,
            roll_right: KeyCode::KeyE,
            up: KeyCode::Space,
            down: KeyCode::ControlLeft,
            boost: KeyCode::ShiftLeft,
            assist: KeyCode::KeyZ,
            cut: KeyCode::KeyX,
            warp: KeyCode::KeyJ,
            view: KeyCode::KeyV,
            board: KeyCode::KeyF,
        }
    }
}

/// Volume options, each from 0.0 to 1.0.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Audio {
    /// Global volume.
    pub master: f32,
    /// Music volume.
    pub music: f32,
    /// Sound effects volume.
    pub effects: f32,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            effects: 0.8,
        }
    }
}

/// World generation options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct WorldPrefs {
    /// Seed of the last world played, if any.
    pub last_seed: Option<u64>,
}
