//! The persisted settings data model.

use abstractions_net::wire::MAX_PLAYERS_DEFAULT;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::audio::Audio;
use super::gameplay::Gameplay;
use super::servers::{SavedServer, default_servers};
use super::ship_keys::ShipKeys;
use crate::update::UpdatePrefs;
use crate::view::ViewPrefs;
use crate::world::RENDER_STEPS;

/// Every user setting, saved to `settings.toml` in the config directory.
#[derive(Resource, Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    /// Camera options: first or third person on foot and in the ship, and the comfort options.
    pub view: ViewPrefs,
    /// Update options: whether the game looks for a newer release at launch.
    pub updates: UpdatePrefs,
    /// Gameplay options: the hitstop.
    pub gameplay: Gameplay,
    /// The name other players see. Empty until the player first opens the multiplayer menu.
    pub player_name: String,
    /// The servers of the multiplayer menu, in the order the player added them.
    pub servers: Vec<SavedServer>,
    /// Most players a world opened to LAN takes, the host included.
    pub host_max_players: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            graphics: Graphics::default(),
            controls: Controls::default(),
            audio: Audio::default(),
            world: WorldPrefs::default(),
            view: ViewPrefs::default(),
            updates: UpdatePrefs::default(),
            gameplay: Gameplay::default(),
            player_name: String::new(),
            servers: default_servers(),
            host_max_players: MAX_PLAYERS_DEFAULT,
        }
    }
}

/// Rendering options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Graphics {
    /// Chunk radius streamed around the player: the rings of one of the named steps of
    /// [`RENDER_STEPS`]. Any other number in a file moves to the nearest step when read.
    #[serde(deserialize_with = "nearest_step")]
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
    /// Vertical field of view in degrees (60 to 110).
    pub fov: u32,
    /// Whether the frame rate is shown in a corner.
    pub show_fps: bool,
    /// Shadow, exposure and reflection options of the light, see `lighting.rs`.
    pub lighting: super::lighting::Lighting,
}

/// Returns the index in [`RENDER_STEPS`] of the step nearest to a detail radius of `rings` chunks; a
/// radius halfway between two steps takes the lower one.
pub fn render_step(rings: u32) -> usize {
    let gap = |(_, step): &(&str, u32)| step.abs_diff(rings);
    (0..RENDER_STEPS.len())
        .min_by_key(|&i| gap(&RENDER_STEPS[i]))
        .unwrap_or(0)
}

/// Reads a render distance and moves it to the nearest named step, so that files written before
/// the steps existed still load.
fn nearest_step<'de, D: serde::Deserializer<'de>>(reader: D) -> Result<u32, D::Error> {
    let rings = u32::deserialize(reader)?;
    Ok(RENDER_STEPS[render_step(rings)].1)
}

/// Reads the key bindings and moves the minimap to N in files written before the map had M:
/// such a file binds the minimap to M and leaves the map on its default M.
fn moved_minimap<'de, D: serde::Deserializer<'de>>(reader: D) -> Result<Keymap, D::Error> {
    let mut keys = Keymap::deserialize(reader)?;
    if keys.minimap == KeyCode::KeyM && keys.map == KeyCode::KeyM {
        keys.minimap = KeyCode::KeyN;
    }
    Ok(keys)
}

impl Default for Graphics {
    fn default() -> Self {
        Self {
            render_distance: 4,
            shadows: true,
            vsync: true,
            fullscreen: false,
            ambient_occlusion: true,
            bloom: true,
            clouds: true,
            fov: 75,
            show_fps: false,
            lighting: Default::default(),
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
    #[serde(deserialize_with = "moved_minimap")]
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
    /// Jump, or swim up in water. Pressed again in the air, it spreads or folds the wingsuit.
    pub jump: KeyCode,
    /// Run faster. Pressed in the air it fires the boot boosters until let go: they brake a
    /// fall with the wingsuit folded and push with it spread. Held from a run, only a fast fall
    /// with the wingsuit folded fires them.
    pub sprint: KeyCode,
    /// Crouch while held: slower, quieter and lower on the ground.
    pub crouch: KeyCode,
    /// Open and close the board computer while seated in the ship.
    pub computer: KeyCode,
    /// Hand the destination of the board computer to the autopilot.
    pub engage: KeyCode,
    /// Spread or fold the wingsuit in the air, as the jump key pressed again there does.
    pub wings: KeyCode,
    /// Switch between first and third person, in the ship and on foot.
    pub view: KeyCode,
    /// Bring the view in the ship back behind the nose, or to the nose from the pilot seat.
    pub recentre: KeyCode,
    /// Step the minimap through its ranges, then off.
    pub minimap: KeyCode,
    /// Open and close the map, on foot and in the ship.
    pub map: KeyCode,
    /// Drop a pin at the player's feet, or remove the pin within ten metres of them.
    pub waypoint: KeyCode,
    /// Open and close the field guide of the planet underfoot.
    pub guide: KeyCode,
    /// Open and close the pack and the workshop, on foot.
    pub pack: KeyCode,
    /// Patch the worst damaged part of the robot with what the pack holds, on foot.
    pub mend: KeyCode,
    /// Held, draws the spear in hand back; let go, throws it. A click of the primary button stabs
    /// with it instead, so a spear never leaves the hand by a click.
    pub throw: KeyCode,
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
            crouch: KeyCode::ControlLeft,
            computer: KeyCode::Tab,
            engage: KeyCode::KeyG,
            wings: KeyCode::KeyR,
            view: KeyCode::KeyV,
            recentre: KeyCode::KeyR,
            minimap: KeyCode::KeyN,
            map: KeyCode::KeyM,
            waypoint: KeyCode::KeyB,
            guide: KeyCode::KeyK,
            pack: KeyCode::KeyI,
            mend: KeyCode::KeyU,
            throw: KeyCode::KeyT,
            ship: ShipKeys::default(),
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
