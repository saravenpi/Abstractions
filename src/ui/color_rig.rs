//! Test rig: contrast of text on the HUD panel over a sky of a given brightness.

use bevy::prelude::*;

use super::backdrop::contrast;
use super::theme;

/// Converts one sRGB channel to linear light.
pub fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Returns the relative luminance of an opaque sRGB colour.
pub fn luminance(c: Srgba) -> f32 {
    0.2126 * linear(c.red) + 0.7152 * linear(c.green) + 0.0722 * linear(c.blue)
}

/// Lays `top` over the opaque colour `under`, as the screen does, channel by channel in sRGB.
pub fn over(top: Color, under: Srgba) -> Srgba {
    let t = top.to_srgba();
    let mix = |a: f32, b: f32| a * t.alpha + b * (1.0 - t.alpha);
    Srgba::new(
        mix(t.red, under.red),
        mix(t.green, under.green),
        mix(t.blue, under.blue),
        1.0,
    )
}

/// Contrast of `text` drawn on `PANEL` over a world of the given brightness.
pub fn on_panel(text: Color, world: f32) -> f32 {
    let sky = Srgba::new(world, world, world, 1.0);
    let panel = over(theme::PANEL, sky);
    contrast(luminance(over(text, panel)), luminance(panel))
}
