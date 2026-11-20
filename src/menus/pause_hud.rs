//! Corner texts of the pause menu: the world name or the session, and the save notice.

use bevy::prelude::*;

use crate::net::Session;
use crate::saves::{ActiveWorld, SavedFlash};
use crate::ui::{text, theme};
use crate::world::WorldSeed;

/// Marks the "Saved" notice.
#[derive(Component)]
pub struct SavedLabel;

/// Returns the corner text: the LAN address and key while hosting, the server while joined, and
/// otherwise the world name and seed, or the seed alone.
pub fn corner_text(
    session: &Session,
    active: Option<&ActiveWorld>,
    seed: Option<&WorldSeed>,
) -> String {
    match (session, active, seed) {
        (Session::Hosting { lan_address, key }, _, _) => {
            format!("Hosting on {lan_address} - key {key}")
        }
        (Session::Joined { address, name }, _, _) => format!("{name} - {address}"),
        (_, Some(active), _) => format!("{} - seed {}", active.0.save.name, active.0.save.seed),
        (_, None, Some(seed)) => format!("Seed {}", seed.0),
        (_, None, None) => String::new(),
    }
}

/// Small muted text in the bottom-right corner.
pub fn world_corner(content: String) -> impl Bundle {
    (
        text(content, theme::FONT_SMALL, theme::TEXT_2),
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
        text("Saved", theme::FONT_SMALL, theme::TEXT_2),
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
