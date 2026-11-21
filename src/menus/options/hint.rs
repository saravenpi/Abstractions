//! The line under the settings: on the graphics tab a note on the render distance steps that need
//! a strong GPU, and the first key conflict on the controls and ship tabs.

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use super::actions::SettingsAction;
use super::binds::conflicts;
use super::describe::{COSTS, describe, describe_general};
use super::field::Field;
use super::names::key_label;
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
        text("", theme::FONT_SMALL, theme::TEXT_2),
        Node {
            margin: UiRect::new(px(theme::INSET), px(0), px(theme::S3), px(0)),
            min_height: px(theme::LINE_SMALL),
            flex_shrink: 0.0,
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
        key_label(first.key(keys)),
        first.label(),
        second.label()
    )
}

/// Returns the line of the graphics tab: the description of the focused row when it has one,
/// else the render distance note, else what costs the most.
fn graphics_hint(rings: u32, focused: Option<Field>) -> String {
    let note = graphics_line(rings);
    let said = focused.map_or("", describe);
    match (said.is_empty(), note.is_empty()) {
        (true, true) => COSTS.to_string(),
        (true, false) => note,
        (false, false) if focused == Some(Field::RenderDistance) => format!("{said} {note}"),
        (false, _) => said.to_string(),
    }
}

/// Shows the description of the focused setting on the graphics tab and the key conflicts on the
/// key tabs.
fn show_hint(
    (tab, focus): (Res<SettingsTab>, Res<InputFocus>),
    settings: Res<Persistent<Settings>>,
    actions: Query<&SettingsAction>,
    mut lines: Query<(&mut Text, &mut TextColor), With<HintLine>>,
) {
    let on = focus.get().and_then(|entity| actions.get(entity).ok());
    let field = match on {
        Some(SettingsAction::Adjust(field, _)) => Some(*field),
        _ => None,
    };
    let (shown, color) = match tab.0 {
        Tab::Graphics => (
            graphics_hint(settings.graphics.render_distance, field),
            theme::TEXT_2,
        ),
        Tab::Controls | Tab::Ship => (conflict_line(&settings.controls.keys), theme::TEXT),
        Tab::General => (
            field.map_or("", describe_general).to_string(),
            theme::TEXT_2,
        ),
        _ => (String::new(), theme::TEXT_2),
    };
    for (mut line, mut tint) in &mut lines {
        line.set_if_neq(Text(shown.clone()));
        tint.set_if_neq(TextColor(color));
    }
}

/// What the hint line depends on.
fn stale(
    (tab, focus): (Res<SettingsTab>, Res<InputFocus>),
    settings: Res<Persistent<Settings>>,
    fresh: Query<(), Added<HintLine>>,
) -> bool {
    tab.is_changed() || focus.is_changed() || settings.is_changed() || !fresh.is_empty()
}

/// Registers the hint system.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        show_hint.run_if(in_state(Menu::Settings)).run_if(stale),
    );
}
