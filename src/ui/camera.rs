//! The overlay camera that draws all UI above the 3D view.

use bevy::camera::CameraOutputMode;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;

use crate::render::GameCamera;

/// Marks the UI overlay camera.
#[derive(Component)]
pub struct UiCamera;

/// Returns how the overlay is written to the window: blended over the world, or over a cleared window.
fn output_mode(world_visible: bool) -> CameraOutputMode {
    CameraOutputMode::Write {
        blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        clear_color: match world_visible {
            true => ClearColorConfig::None,
            false => ClearColorConfig::Default,
        },
    }
}

/// Spawns the overlay camera. It clears its own layer every frame so no menu ever lingers.
pub(super) fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        UiCamera,
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            output_mode: output_mode(false),
            ..default()
        },
        Msaa::Sample4,
        IsDefaultUiCamera,
    ));
}

/// Clears the window behind the overlay only while no world camera draws there.
pub(super) fn sync_output(
    worlds: Query<(), With<GameCamera>>,
    mut overlay: Single<&mut Camera, With<UiCamera>>,
    mut seen: Local<Option<bool>>,
) {
    let world_visible = !worlds.is_empty();
    if *seen != Some(world_visible) {
        *seen = Some(world_visible);
        overlay.output_mode = output_mode(world_visible);
    }
}
