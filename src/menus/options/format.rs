//! Display text of individual settings.

use bevy::prelude::KeyCode;

use super::field::Field;
use crate::settings::{Graphics, Settings};
use crate::ui::theme;

/// Formats a boolean as On or Off.
pub fn on_off(value: bool) -> String {
    if value { "On" } else { "Off" }.to_string()
}

/// Returns the value of a graphics toggle, if the field is one.
fn flag_value(g: &Graphics, field: Field) -> Option<bool> {
    match field {
        Field::Shadows => Some(g.shadows),
        Field::Vsync => Some(g.vsync),
        Field::Fullscreen => Some(g.fullscreen),
        Field::AmbientOcclusion => Some(g.ambient_occlusion),
        Field::Bloom => Some(g.bloom),
        Field::Clouds => Some(g.clouds),
        Field::ShowFps => Some(g.show_fps),
        Field::RayTracing => Some(g.ray_tracing),
        _ => None,
    }
}

/// Formats a graphics setting, or `None` when the field is not a graphics one.
fn display_graphics(g: &Graphics, field: Field) -> Option<String> {
    if let Some(value) = flag_value(g, field) {
        return Some(on_off(value));
    }
    match field {
        Field::PixelSize => Some(g.pixel_size.to_string()),
        Field::RenderDistance => Some(g.render_distance.to_string()),
        Field::ColorLevels => Some(g.color_levels.to_string()),
        Field::Fov => Some(format!("{} deg", g.fov)),
        _ => None,
    }
}

/// Formats the current value of a setting for display.
pub fn display(settings: &Settings, field: Field) -> String {
    let (c, a) = (&settings.controls, &settings.audio);
    if let Some(shown) = display_graphics(&settings.graphics, field) {
        return shown;
    }
    match field {
        Field::Sensitivity => format!("{:.1}", c.mouse_sensitivity),
        Field::InvertY => on_off(c.invert_y),
        Field::Master => format!("{:.1}", a.master),
        Field::Music => format!("{:.1}", a.music),
        Field::Effects => format!("{:.1}", a.effects),
        Field::Key(bind) => key_name(bind.key(&c.keys)),
        _ => String::new(),
    }
}

/// Most characters of a key name: the value column less the two of the conflict mark.
pub const KEY_CHARS: usize = theme::VALUE_CHARS - 2;

/// Readable names of keys whose code name is long or unclear.
const NAMES: [(KeyCode, &str); 12] = [
    (KeyCode::ShiftLeft, "Left Shift"),
    (KeyCode::ShiftRight, "Right Shift"),
    (KeyCode::ControlLeft, "Left Ctrl"),
    (KeyCode::ControlRight, "Right Ctrl"),
    (KeyCode::AltLeft, "Left Alt"),
    (KeyCode::AltRight, "Right Alt"),
    (KeyCode::ArrowUp, "Up"),
    (KeyCode::ArrowDown, "Down"),
    (KeyCode::ArrowLeft, "Left"),
    (KeyCode::ArrowRight, "Right"),
    (KeyCode::Backquote, "`"),
    (KeyCode::NumpadEnter, "Num Enter"),
];

/// Returns a short readable name for a key, never longer than the value column holds.
pub fn key_name(key: KeyCode) -> String {
    if let Some((_, name)) = NAMES.iter().find(|(code, _)| *code == key) {
        return name.to_string();
    }
    let name = format!("{key:?}");
    let short = ["Key", "Digit"]
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix));
    let short = short.unwrap_or(&name).replace("Numpad", "Num ");
    short.chars().take(KEY_CHARS).collect()
}
