//! Layout and live values of the settings menu.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use super::actions::SettingsAction;
use super::field::Field;
use super::format::display;
use super::hint;
use super::pages::{self, Page, display_for};
use super::rebind::Rebinding;
use super::rows::Value;
use super::{SettingsTab, Tab};
use crate::menus::{Menu, Nav};
use crate::settings::{Persistent, Settings};
use crate::ui::{Tone, caps, column, footer, item_with, label, root, theme};

/// Marks the text of a tab item so its tone follows the active tab.
#[derive(Component)]
struct TabLabel(Tab);

/// Returns the tone of a tab item.
fn tab_tone(tab: Tab, active: Tab) -> Tone {
    if tab == active {
        Tone::Accent
    } else {
        Tone::Plain
    }
}

/// One tab item in the left column.
fn tab_item(name: &str, tab: Tab, active: Tab) -> impl Bundle {
    (
        item_with(
            0,
            (
                label(name, theme::FONT_BODY, tab_tone(tab, active)),
                TabLabel(tab),
            ),
        ),
        SettingsAction::Tab(tab),
    )
}

/// Left column with the tabs on top and Back at the bottom.
fn sidebar(active: Tab) -> impl Bundle {
    (
        Node {
            width: px(theme::MENU_WIDTH * 0.6),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        children![
            (
                column(percent(100), 0.0),
                children![
                    (tab_item("Graphics", Tab::Graphics, active), AutoFocus),
                    tab_item("Controls", Tab::Controls, active),
                    tab_item("Audio", Tab::Audio, active),
                ]
            ),
            (
                item_with(2, label("Back", theme::FONT_BODY, Tone::Plain)),
                Nav::Back
            ),
        ],
    )
}

/// Sidebar and the three pages side by side.
fn body(active: Tab) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(theme::ROW_HEIGHT * 12.0),
            column_gap: px(theme::SPACE * 8.0),
            ..default()
        },
        children![
            sidebar(active),
            (
                column(px(theme::MENU_WIDTH * 2.0), 0.0),
                children![
                    pages::graphics(active),
                    pages::controls(active),
                    pages::audio(active),
                ]
            ),
        ],
    )
}

/// Spawns the whole settings menu.
fn spawn(mut commands: Commands, tab: Res<SettingsTab>) {
    commands.spawn((
        DespawnOnExit(Menu::Settings),
        root(),
        children![
            (
                caps("Settings"),
                Node {
                    margin: UiRect::bottom(px(theme::SPACE * 4.0)),
                    ..default()
                }
            ),
            body(tab.0),
            hint::line(),
            footer("Esc to go back. Left and right change a value"),
        ],
    ));
}

/// Shows only the page of the active tab and marks the active tab item.
fn show_tab(
    tab: Res<SettingsTab>,
    mut pages: Query<(&Page, &mut Node)>,
    mut labels: Query<(&TabLabel, &mut Tone)>,
) {
    for (page, mut node) in &mut pages {
        node.display = display_for(page.0, tab.0);
    }
    for (label, mut tone) in &mut labels {
        tone.set_if_neq(tab_tone(label.0, tab.0));
    }
}

/// Writes the current setting values into their texts and colors toggles.
fn show_values(
    settings: Res<Persistent<Settings>>,
    rebinding: Res<Rebinding>,
    mut values: Query<(&Value, &mut Text, &mut Tone)>,
) {
    for (value, mut text, mut tone) in &mut values {
        let shown = match (value.0, rebinding.0) {
            (Field::Key(bind), Some(waiting)) if bind == waiting => "...".to_string(),
            (field, _) => display(&settings, field),
        };
        tone.set_if_neq(if shown == "On" {
            Tone::Accent
        } else {
            Tone::Plain
        });
        text.set_if_neq(Text(shown));
    }
}

/// Registers the settings menu view systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn).add_systems(
        Update,
        (
            show_tab.run_if(resource_changed::<SettingsTab>),
            show_values,
        )
            .run_if(in_state(Menu::Settings)),
    );
}
