//! Display text of individual settings.

use bevy::prelude::KeyCode;

use super::field::{Bind, Field};
use crate::settings::{Graphics, Settings};

/// Returns the key currently bound to an action.
fn key_of(settings: &Settings, bind: Bind) -> KeyCode {
    let keys = &settings.controls.keys;
    match bind {
        Bind::Forward => keys.forward,
        Bind::Back => keys.back,
        Bind::Left => keys.left,
        Bind::Right => keys.right,
        Bind::Jump => keys.jump,
        Bind::Sprint => keys.sprint,
    }
}

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
        Field::Key(bind) => key_name(key_of(settings, bind)),
        _ => String::new(),
    }
}

/// Returns a short readable name for a key.
pub fn key_name(key: KeyCode) -> String {
    let name = format!("{key:?}");
    let short = name
        .strip_prefix("Key")
        .or_else(|| name.strip_prefix("Digit"));
    short.unwrap_or(&name).to_string()
}
