//! The overlay camera that draws all UI above the low-resolution 3D view.

use bevy::camera::CameraOutputMode;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;

use crate::render::{GameCamera, LowRes};

/// Marks the UI overlay camera.
#[derive(Component)]
pub struct UiCamera;

/// Marks the full-window image that shows the game camera behind all UI.
#[derive(Component)]
pub struct WorldView;

/// Spawns the overlay camera and the world view. The camera clears the window and its own layer every frame.
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
        Msaa::Sample4,
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
