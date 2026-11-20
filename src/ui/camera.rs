//! The overlay camera that draws all UI above the low-resolution 3D view.

use bevy::camera::CameraOutputMode;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy::window::PrimaryWindow;

use crate::render::{Fit, GameCamera, LowRes};

/// Marks the UI overlay camera.
#[derive(Component)]
pub struct UiCamera;

/// Marks the full-window image that shows the game camera behind all UI.
#[derive(Component)]
pub struct WorldView;

/// Spawns the overlay camera and the world view. The camera clears the window and its own layer every frame.
///
/// The camera draws without multisampling: it draws only the interface and the game image, which
/// need none, and a multisampled target at window size would cost four samples per window pixel.
pub(super) fn spawn_camera(mut commands: Commands, low: Res<LowRes>) {
    commands.spawn((
        UiCamera,
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                clear_color: ClearColorConfig::Default,
            },
            ..default()
        },
        Msaa::Off,
        IsDefaultUiCamera,
    ));
    commands.spawn((
        WorldView,
        ImageNode::new(low.0.clone()),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(i32::MIN),
        Pickable::IGNORE,
        Visibility::Hidden,
    ));
}

/// Shows the world view only while a game camera renders into it.
pub(super) fn sync_output(
    worlds: Query<(), With<GameCamera>>,
    mut view: Single<&mut Visibility, With<WorldView>>,
) {
    let wanted = match worlds.is_empty() {
        true => Visibility::Hidden,
        false => Visibility::Inherited,
    };
    view.set_if_neq(wanted);
}

/// Window height in logical pixels the interface is designed for.
pub const REFERENCE_HEIGHT: f32 = 720.0;

/// Returns the interface scale for a window `height` logical pixels high: 1 at the 720 pixel
/// reference, 0.75 at the smallest supported window, 3 at most.
pub fn ui_scale_for(height: f32) -> f32 {
    (height / REFERENCE_HEIGHT).clamp(0.75, 3.0)
}

/// Shows the game image at its whole scale: the part of it that fits the window, centred, with
/// every game pixel the same square of window pixels.
pub(super) fn frame_view(
    window: Single<&Window, With<PrimaryWindow>>,
    mut view: Single<&mut ImageNode, With<WorldView>>,
    mut scale: ResMut<UiScale>,
) {
    let wanted = ui_scale_for(window.height());
    if scale.0 != wanted {
        scale.0 = wanted;
    }
    let physical = window.physical_size();
    let shown = Some(Fit::of(physical).shown(physical));
    if view.rect != shown {
        view.rect = shown;
    }
}
