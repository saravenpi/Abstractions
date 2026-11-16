//! The muted line under the graphics settings that explains what ray tracing needs.

use bevy::prelude::*;

use super::{SettingsTab, Tab};
use crate::menus::Menu;
use crate::render::ray_tracing::{RayTracingBoot, hint};
use crate::settings::{Persistent, Settings};
use crate::ui::{text, theme};

/// Marks the text of the hint line.
#[derive(Component)]
pub struct HintLine;

/// The empty hint line, filled in by [`show_hint`].
pub fn line() -> impl Bundle {
    (
        HintLine,
        text("", theme::FONT_SMALL, theme::MUTED),
        Node {
            margin: UiRect::top(px(theme::SPACE)),
            ..default()
        },
    )
}

/// Shows the ray tracing hint on the graphics tab and nothing elsewhere.
fn show_hint(
    settings: Res<Persistent<Settings>>,
    boot: Res<RayTracingBoot>,
    tab: Res<SettingsTab>,
    mut lines: Query<&mut Text, With<HintLine>>,
) {
    let shown = match tab.0 {
        Tab::Graphics => hint(settings.graphics.ray_tracing, *boot).unwrap_or_default(),
        _ => "",
    };
    for mut line in &mut lines {
        line.set_if_neq(Text::new(shown));
    }
}

/// Registers the hint system.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, show_hint.run_if(in_state(Menu::Settings)));
}
