//! The line under the settings: on the graphics tab a note on the render distance steps that need
//! a strong GPU, and the first key conflict on the controls and ship tabs.

use bevy::prelude::*;

use super::binds::conflicts;
use super::format::key_name;
use super::{SettingsTab, Tab};
use crate::menus::Menu;
use crate::settings::{Keymap, Persistent, Settings, render_step};
use crate::ui::{text, theme};
use crate::world::RENDER_STEPS;

/// Marks the text of the hint line.
#[derive(Component)]
pub struct HintLine;

/// The empty hint line, filled in by [`show_hint`].
pub fn line() -> impl Bundle {
    (
        HintLine,
        text("", theme::FONT_SMALL, theme::MUTED),
        Node {
            margin: UiRect::top(px(theme::GAP_S)),
            min_height: px(theme::GAP_L),
            ..default()
        },
    )
}

/// First step of the render distance, in [`RENDER_STEPS`], that asks for a strong GPU: Ultra.
const STRONG_GPU: usize = 3;

/// Returns the line of the graphics tab: a note when the render distance of `rings` chunks is
/// one of the steps that need a strong GPU, and nothing otherwise.
pub fn graphics_line(rings: u32) -> String {
    let step = render_step(rings);
    match step >= STRONG_GPU {
        true => format!("{} needs a strong GPU.", RENDER_STEPS[step].0),
        false => String::new(),
    }
}

/// Returns the sentence that names the first key conflict, or nothing when there is none.
pub fn conflict_line(keys: &Keymap) -> String {
    let found = conflicts(keys);
    let Some((first, second)) = found.first() else {
        return String::new();
    };
    let more = match found.len() - 1 {
        0 => String::new(),
        n => format!(", and {n} more"),
    };
    format!(
        "{} is bound to both \"{}\" and \"{}\"{more}",
        key_name(first.key(keys)),
        first.label(),
        second.label()
    )
}

/// Shows the render distance note on the graphics tab and the key conflicts on the key tabs.
fn show_hint(
    tab: Res<SettingsTab>,
    settings: Res<Persistent<Settings>>,
    mut lines: Query<(&mut Text, &mut TextColor), With<HintLine>>,
) {
    let (shown, color) = match tab.0 {
        Tab::Graphics => (
            graphics_line(settings.graphics.render_distance),
            theme::MUTED,
        ),
        Tab::Controls | Tab::Ship => (conflict_line(&settings.controls.keys), theme::TEXT),
        _ => (String::new(), theme::MUTED),
    };
    for (mut line, mut tint) in &mut lines {
        line.set_if_neq(Text(shown.clone()));
        tint.set_if_neq(TextColor(color));
    }
}

/// What the hint line depends on.
fn stale(
    tab: Res<SettingsTab>,
    settings: Res<Persistent<Settings>>,
    fresh: Query<(), Added<HintLine>>,
) -> bool {
    tab.is_changed() || settings.is_changed() || !fresh.is_empty()
}

/// Registers the hint system.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        show_hint.run_if(in_state(Menu::Settings)).run_if(stale),
    );
}
