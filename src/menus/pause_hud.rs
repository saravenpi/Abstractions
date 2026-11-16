//! Corner texts of the pause menu: the world name and the save notice.

use bevy::prelude::*;

use crate::saves::SavedFlash;
use crate::ui::{text, theme};

/// Marks the "Saved" notice.
#[derive(Component)]
pub struct SavedLabel;

/// Small muted text in the bottom-right corner.
pub fn world_corner(content: String) -> impl Bundle {
    (
        text(content, theme::FONT_SMALL, theme::MUTED),
        Node {
            position_type: PositionType::Absolute,
            right: px(theme::EDGE),
            bottom: px(theme::EDGE),
            ..default()
        },
    )
}

/// Small muted "Saved" in the top-right corner, hidden until a save happens.
pub fn saved_label() -> impl Bundle {
    (
        SavedLabel,
        text("Saved", theme::FONT_SMALL, theme::MUTED),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            right: px(theme::EDGE),
            top: px(theme::EDGE),
            ..default()
        },
    )
}

/// Shows the notice while the save flash lasts.
pub fn show_saved(flash: Res<SavedFlash>, mut labels: Query<&mut Visibility, With<SavedLabel>>) {
    let wanted = if flash.0 > 0.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut labels {
        visibility.set_if_neq(wanted);
    }
}
