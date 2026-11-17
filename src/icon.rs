//! The window icon: a white square centred on a black tile, generated in code and set on the
//! window once it exists.
//!
//! The proportions are those of the 64 px image of `packaging/windows/abstractions.ico`.

#[cfg(not(target_os = "macos"))]
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
#[cfg(not(target_os = "macos"))]
use bevy::window::PrimaryWindow;
#[cfg(not(target_os = "macos"))]
use bevy::winit::WINIT_WINDOWS;
#[cfg(not(target_os = "macos"))]
use winit::window::Icon;

/// Width and height of the window icon in pixels.
pub const SIZE: u32 = 64;

/// Width of the white square in pixels: 42 percent of the tile, rounded so it centres on whole pixels.
pub const SQUARE: u32 = 26;

/// Black on each side of the white square in pixels.
pub const PADDING: u32 = (SIZE - SQUARE) / 2;

/// Smallest share of the tile width left black on each side of the square.
pub const PADDING_SHARE: f32 = 0.28;

/// RGBA bytes of the icon, row by row from the top.
pub fn pixels() -> Vec<u8> {
    let inside = |at: u32| (PADDING..PADDING + SQUARE).contains(&at);
    (0..SIZE * SIZE)
        .flat_map(|index| {
            let shade = if inside(index % SIZE) && inside(index / SIZE) {
                255
            } else {
                0
            };
            [shade, shade, shade, 255]
        })
        .collect()
}

/// Sets the icon on the primary window once winit has made it, then stops.
///
/// Nothing happens without a window, as in a headless run, and on macOS, where the bundle
/// carries the icon and winit has no window icon.
#[cfg(not(target_os = "macos"))]
fn set_icon(
    _main_thread: NonSendMarker,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Ok(entity) = windows.single() else {
        return;
    };
    let Ok(icon) = Icon::from_rgba(pixels(), SIZE, SIZE) else {
        *done = true;
        return;
    };
    WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(window) = windows.get_window(entity) {
            window.set_window_icon(Some(icon));
            *done = true;
        }
    });
}

/// Registers the system that sets the window icon.
pub fn plugin(app: &mut App) {
    #[cfg(not(target_os = "macos"))]
    app.add_systems(Update, set_icon);
    #[cfg(target_os = "macos")]
    let _ = app;
}

#[cfg(test)]
mod tests;
