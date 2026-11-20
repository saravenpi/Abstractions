//! The interaction prompt: a key and what it does, shown only while it can be done, and over
//! it in the ship the action line of the course.

mod course;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::menus::key_name;
use crate::settings::{Persistent, Settings};
use crate::ship::{Hatch, ShipTelemetry};
use crate::space::{Location, Travel};
use crate::ui::{Tone, key_cap, label, theme};

/// Marks the prompt row.
#[derive(Component)]
pub(super) struct Prompt;

/// Marks the text that names the action of the prompt.
#[derive(Component)]
pub(super) struct PromptAction;

/// Marks the text that names the key of the prompt.
#[derive(Component)]
pub(super) struct PromptKey;

/// Returns what the board key does right now, or nothing when it does nothing: boarding from
/// the hatch of the landed ship, or leaving the landed ship.
pub fn board_prompt(travel: Travel, at_hatch: bool, landed: bool) -> Option<&'static str> {
    match travel {
        Travel::OnFoot if at_hatch => Some("Board"),
        Travel::InShip if landed => Some("Leave the ship"),
        _ => None,
    }
}

/// The hidden prompt, low in the middle of the screen, under the hidden action line of the
/// course. `update` fills both.
pub(super) fn line() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            bottom: percent(22),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(theme::GAP_S),
            ..default()
        },
        children![
            course::row(),
            (
                Prompt,
                Node {
                    display: Display::None,
                    align_items: AlignItems::Center,
                    column_gap: px(theme::GAP_S),
                    padding: UiRect::axes(px(theme::GAP_S), px(theme::GAP_XS)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                children![
                    (PromptKey, key_cap("")),
                    (PromptAction, label("", theme::FONT_SMALL, Tone::Plain)),
                ],
            )
        ],
    )
}

/// What decides the prompt.
#[derive(SystemParam)]
pub(super) struct Reach<'w> {
    location: Option<Res<'w, Location>>,
    hatch: Option<Res<'w, Hatch>>,
    telemetry: Option<Res<'w, ShipTelemetry>>,
    settings: Res<'w, Persistent<Settings>>,
}

impl Reach<'_> {
    /// Returns the action of the board key and the key itself, when the key does something.
    fn prompt(&self) -> Option<(&'static str, KeyCode)> {
        let travel = self.location.as_deref()?.travel;
        let at_hatch = self.hatch.as_deref().is_some_and(|hatch| hatch.near);
        let landed = self.telemetry.as_deref().is_some_and(|ship| ship.landed);
        let action = board_prompt(travel, at_hatch, landed)?;
        Some((action, self.settings.controls.keys.ship.board))
    }
}

/// The two texts of the prompt: the action and the key.
type Texts<'w, 's> = (
    Query<'w, 's, &'static mut Text, (With<PromptAction>, Without<PromptKey>)>,
    Query<'w, 's, &'static mut Text, (With<PromptKey>, Without<PromptAction>)>,
);

/// Shows the prompt while the board key does something, and rewrites it only when it changes
/// or when the HUD was just spawned. Writes the action line of the course too.
pub(super) fn update(
    (reach, mut line): (Reach, course::Line),
    mut shown: Local<Option<(&'static str, KeyCode)>>,
    fresh: Query<(), Added<Prompt>>,
    mut rows: Query<&mut Node, (With<Prompt>, Without<course::CourseRow>)>,
    (mut actions, mut keys): Texts,
) {
    line.write();
    let wanted = reach.prompt();
    if wanted == *shown && fresh.is_empty() {
        return;
    }
    *shown = wanted;
    for mut node in &mut rows {
        node.display = match wanted {
            Some(_) => Display::Flex,
            None => Display::None,
        };
    }
    let Some((action, key)) = wanted else {
        return;
    };
    for mut text in &mut actions {
        text.set_if_neq(Text(action.to_string()));
    }
    for mut text in &mut keys {
        text.set_if_neq(Text(key_name(key)));
    }
}
