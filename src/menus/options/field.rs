//! Pure editing and formatting of individual settings.

use bevy::prelude::KeyCode;

use crate::settings::Settings;

/// One rebindable player action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bind {
    Forward,
    Back,
    Left,
    Right,
    Jump,
    Sprint,
}

/// One editable setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    PixelSize,
    RenderDistance,
    Shadows,
    Vsync,
    Fullscreen,
    Sensitivity,
    InvertY,
    Master,
    Music,
    Effects,
    Key(Bind),
}

/// Moves a whole number by `dir` inside `lo..=hi`.
fn bump_int(value: u32, dir: i32, lo: u32, hi: u32) -> u32 {
    value.saturating_add_signed(dir).clamp(lo, hi)
}

/// Moves a value by `dir` tenths inside `lo..=hi`.
fn bump_tenths(value: f32, dir: i32, lo: f32, hi: f32) -> f32 {
    (((value * 10.0).round() + dir as f32) / 10.0).clamp(lo, hi)
}

/// Steps a setting by `dir`. Toggles flip, numbers move and clamp, keys are untouched.
pub fn step(settings: &mut Settings, field: Field, dir: i32) {
    let (g, c, a) = (
        &mut settings.graphics,
        &mut settings.controls,
        &mut settings.audio,
    );
    match field {
        Field::PixelSize => g.pixel_size = bump_int(g.pixel_size, dir, 1, 8),
        Field::RenderDistance => g.render_distance = bump_int(g.render_distance, dir, 2, 8),
        Field::Shadows => g.shadows = !g.shadows,
        Field::Vsync => g.vsync = !g.vsync,
        Field::Fullscreen => g.fullscreen = !g.fullscreen,
        Field::Sensitivity => c.mouse_sensitivity = bump_tenths(c.mouse_sensitivity, dir, 0.1, 3.0),
        Field::InvertY => c.invert_y = !c.invert_y,
        Field::Master => a.master = bump_tenths(a.master, dir, 0.0, 1.0),
        Field::Music => a.music = bump_tenths(a.music, dir, 0.0, 1.0),
        Field::Effects => a.effects = bump_tenths(a.effects, dir, 0.0, 1.0),
        Field::Key(_) => {}
    }
}

/// Assigns a key to an action.
pub fn set_key(settings: &mut Settings, bind: Bind, key: KeyCode) {
    let keys = &mut settings.controls.keys;
    match bind {
        Bind::Forward => keys.forward = key,
        Bind::Back => keys.back = key,
        Bind::Left => keys.left = key,
        Bind::Right => keys.right = key,
        Bind::Jump => keys.jump = key,
        Bind::Sprint => keys.sprint = key,
    }
}

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
fn on_off(value: bool) -> String {
    if value { "On" } else { "Off" }.to_string()
}

/// Formats the current value of a setting for display.
pub fn display(settings: &Settings, field: Field) -> String {
    let (g, c, a) = (&settings.graphics, &settings.controls, &settings.audio);
    match field {
        Field::PixelSize => g.pixel_size.to_string(),
        Field::RenderDistance => g.render_distance.to_string(),
        Field::Shadows => on_off(g.shadows),
        Field::Vsync => on_off(g.vsync),
        Field::Fullscreen => on_off(g.fullscreen),
        Field::Sensitivity => format!("{:.1}", c.mouse_sensitivity),
        Field::InvertY => on_off(c.invert_y),
        Field::Master => format!("{:.1}", a.master),
        Field::Music => format!("{:.1}", a.music),
        Field::Effects => format!("{:.1}", a.effects),
        Field::Key(bind) => key_name(key_of(settings, bind)),
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
