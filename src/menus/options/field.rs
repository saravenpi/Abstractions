//! Pure editing of individual settings.

use bevy::prelude::KeyCode;

pub use super::binds::Bind;
use crate::settings::{Graphics, Settings};

/// One editable setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    PixelSize,
    RenderDistance,
    Shadows,
    Vsync,
    Fullscreen,
    AmbientOcclusion,
    Bloom,
    Clouds,
    ShowFps,
    RayTracing,
    ColorLevels,
    Fov,
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

/// Returns the on/off flag a graphics field edits, if it is a toggle.
fn flag(graphics: &mut Graphics, field: Field) -> Option<&mut bool> {
    match field {
        Field::Shadows => Some(&mut graphics.shadows),
        Field::Vsync => Some(&mut graphics.vsync),
        Field::Fullscreen => Some(&mut graphics.fullscreen),
        Field::AmbientOcclusion => Some(&mut graphics.ambient_occlusion),
        Field::Bloom => Some(&mut graphics.bloom),
        Field::Clouds => Some(&mut graphics.clouds),
        Field::ShowFps => Some(&mut graphics.show_fps),
        Field::RayTracing => Some(&mut graphics.ray_tracing),
        _ => None,
    }
}

/// Steps a graphics setting by `dir`. Toggles flip and numbers move and clamp.
fn step_graphics(g: &mut Graphics, field: Field, dir: i32) {
    if let Some(flag) = flag(g, field) {
        *flag = !*flag;
        return;
    }
    match field {
        Field::PixelSize => g.pixel_size = bump_int(g.pixel_size, dir, 1, 8),
        Field::RenderDistance => g.render_distance = bump_int(g.render_distance, dir, 2, 8),
        Field::ColorLevels => g.color_levels = bump_int(g.color_levels, dir * 4, 8, 64),
        Field::Fov => g.fov = bump_int(g.fov, dir * 5, 60, 110),
        _ => {}
    }
}

/// Steps a setting by `dir`. Toggles flip, numbers move and clamp, keys are untouched.
pub fn step(settings: &mut Settings, field: Field, dir: i32) {
    let (c, a) = (&mut settings.controls, &mut settings.audio);
    step_graphics(&mut settings.graphics, field, dir);
    match field {
        Field::Sensitivity => c.mouse_sensitivity = bump_tenths(c.mouse_sensitivity, dir, 0.1, 3.0),
        Field::InvertY => c.invert_y = !c.invert_y,
        Field::Master => a.master = bump_tenths(a.master, dir, 0.0, 1.0),
        Field::Music => a.music = bump_tenths(a.music, dir, 0.0, 1.0),
        Field::Effects => a.effects = bump_tenths(a.effects, dir, 0.0, 1.0),
        _ => {}
    }
}

/// Assigns a key to an action.
pub fn set_key(settings: &mut Settings, bind: Bind, key: KeyCode) {
    *bind.slot(&mut settings.controls.keys) = key;
}
