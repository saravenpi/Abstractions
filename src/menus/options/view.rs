//! Layout and live values of the settings menu.

use bevy::prelude::*;

use super::actions::SettingsAction;
use super::field::{Field, display};
use super::pages::{self, Page, display_for};
use super::rebind::Rebinding;
use super::rows::Value;
use super::{SettingsTab, Tab};
use crate::menus::{Menu, Nav};
use crate::settings::{Persistent, Settings};
use crate::ui::{button, heading, panel, root, row, theme};

/// Tab selector buttons.
fn tab_bar() -> impl Bundle {
    (
        row(),
        children![
            (
                button("Graphics", 140.0),
                SettingsAction::Tab(Tab::Graphics)
            ),
            (
                button("Controls", 140.0),
                SettingsAction::Tab(Tab::Controls)
            ),
            (button("Audio", 140.0), SettingsAction::Tab(Tab::Audio)),
        ],
    )
}

/// Spawns the whole settings menu.
fn spawn(mut commands: Commands, tab: Res<SettingsTab>) {
    let active = tab.0;
    commands.spawn((
        DespawnOnExit(Menu::Settings),
        root(),
        children![(
            panel(),
            children![
                heading("Settings", theme::FONT_HEADING),
                tab_bar(),
                pages::graphics(active),
                pages::controls(active),
                pages::audio(active),
                (button("Back", theme::BUTTON_WIDTH), Nav::Back),
            ]
        )],
    ));
}

/// Shows only the page of the active tab.
fn show_tab(tab: Res<SettingsTab>, mut pages: Query<(&Page, &mut Node)>) {
    for (page, mut node) in &mut pages {
        node.display = display_for(page.0, tab.0);
    }
}

/// Writes the current setting values into their texts.
fn show_values(
    settings: Res<Persistent<Settings>>,
    rebinding: Res<Rebinding>,
    mut values: Query<(&Value, &mut Text)>,
) {
    for (value, mut text) in &mut values {
        let shown = match (value.0, rebinding.0) {
            (Field::Key(bind), Some(waiting)) if bind == waiting => "...".to_string(),
            (field, _) => display(&settings, field),
        };
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
