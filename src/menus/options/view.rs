//! Layout and live values of the settings menu.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::actions::SettingsAction;
use super::binds::conflicted;
use super::field::Field;
use super::format::display;
use super::pages::{self, Page, display_for};
use super::rebind::Rebinding;
use super::rows::Value;
use super::{SettingsTab, Tab, hint, reference};
use crate::menus::{Menu, Nav};
use crate::settings::{Persistent, Settings};
use crate::ui::{
    Cancels, Icon, Scroller, Selected, Tone, column, entry, footer, heading, item_with, panel,
    root, scroller, theme,
};

/// Key hints shown at the bottom of the screen.
const HINT: &str =
    "Up and down move. Left and right change a value. Enter rebinds a key. Esc goes back";

/// Rows of a page visible at once. A longer page scrolls.
const VISIBLE_ROWS: f32 = 16.0;

/// Fewest rows of a page visible at once, whatever the size of the interface.
pub const MIN_ROWS: f32 = 6.0;

/// Height in logical pixels of everything in the settings screen but the rows of the page: the
/// margins, the panel padding, the heading, the line under the rows and the key hints.
pub const CHROME: f32 = 240.0;

/// Marks the column that scrolls the pages of the settings.
#[derive(Component)]
struct Pages;

/// Returns how many rows of a page show at once in a window `height` logical pixels high: as
/// many as fit, from [`MIN_ROWS`] to [`VISIBLE_ROWS`], so the page scrolls instead of running
/// off the screen when the interface is large.
pub fn rows_for(height: f32) -> f32 {
    ((height - CHROME) / theme::ROW_HEIGHT)
        .floor()
        .clamp(MIN_ROWS, VISIBLE_ROWS)
}

/// One tab item in the left column. The open tab is selected, which is not the keyboard focus.
fn tab_item(tab: Tab, active: Tab) -> impl Bundle {
    let (symbol, name) = tab.face();
    (
        item_with(0, entry(symbol, name, theme::FONT_BODY, Tone::Plain)),
        Selected(tab == active),
        SettingsAction::Tab(tab),
    )
}

/// Left column with the tabs on top and Back at the bottom.
fn sidebar(active: Tab) -> impl Bundle {
    let tabs = SpawnWith(move |parent: &mut ChildSpawner| {
        for tab in Tab::ALL {
            let mut item = parent.spawn(tab_item(tab, active));
            if tab == active {
                item.insert(AutoFocus);
            }
        }
    });
    (
        Node {
            width: px(theme::MENU_WIDTH * 0.6),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: px(theme::GAP_M),
            ..default()
        },
        children![
            (column(percent(100), 0.0), Children::spawn(tabs)),
            (
                item_with(2, entry(Icon::Back, "Back", theme::FONT_BODY, Tone::Plain)),
                Nav::Back,
                Cancels
            ),
        ],
    )
}

/// Sidebar and the pages side by side. The pages share one scrolling column.
fn body(active: Tab) -> impl Bundle {
    (
        Node {
            column_gap: px(theme::GAP_XL),
            ..default()
        },
        children![
            sidebar(active),
            (
                column(px(theme::MENU_WIDTH * 2.0), 0.0),
                children![(
                    (Pages, scroller(theme::ROW_HEIGHT * VISIBLE_ROWS)),
                    children![
                        pages::graphics(active),
                        pages::controls(active),
                        pages::ship(active),
                        pages::audio(active),
                        pages::general(active),
                        reference::help(active),
                    ]
                )]
            ),
        ],
    )
}

/// Spawns the whole settings menu on the tab used last.
fn spawn(mut commands: Commands, tab: Res<SettingsTab>) {
    commands.spawn((
        DespawnOnExit(Menu::Settings),
        root(),
        children![
            (
                panel(),
                children![heading("Settings"), body(tab.0), hint::line()]
            ),
            footer(HINT),
        ],
    ));
}

/// Shows only the page of the active tab, marks the active tab item and scrolls back to the top.
fn show_tab(
    tab: Res<SettingsTab>,
    mut pages: Query<(&Page, &mut Node)>,
    mut items: Query<(&SettingsAction, &mut Selected)>,
    mut scrolls: Query<&mut ScrollPosition, With<Scroller>>,
) {
    for (page, mut node) in &mut pages {
        node.display = display_for(page.0, tab.0);
    }
    for (action, mut selected) in &mut items {
        if let SettingsAction::Tab(own) = action {
            selected.set_if_neq(Selected(*own == tab.0));
        }
    }
    for mut scroll in &mut scrolls {
        scroll.y = 0.0;
    }
}

/// Sets how tall the page column is for the room the window has at the current interface scale.
fn fit_page(
    window: Single<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    mut pages: Query<&mut Node, With<Pages>>,
) {
    let wanted = px(theme::ROW_HEIGHT * rows_for(window.height() / scale.0.max(f32::EPSILON)));
    for mut node in &mut pages {
        if node.max_height != wanted {
            node.max_height = wanted;
        }
    }
}

/// Writes the current setting values into their texts. A value that is off is dimmed, and a key
/// in conflict with another is followed by an exclamation mark. Does nothing while nothing changed.
fn show_values(
    settings: Res<Persistent<Settings>>,
    rebinding: Res<Rebinding>,
    fresh: Query<(), Added<Value>>,
    mut values: Query<(&Value, &mut Text, &mut Tone)>,
) {
    if !(settings.is_changed() || rebinding.is_changed() || !fresh.is_empty()) {
        return;
    }
    let keys = &settings.controls.keys;
    for (value, mut text, mut tone) in &mut values {
        let (shown, wanted) = match value.0 {
            Field::Key(bind) if rebinding.bind() == Some(bind) => ("...".to_string(), Tone::Plain),
            Field::Key(bind) if conflicted(keys, bind) => {
                (format!("{} !", display(&settings, value.0)), Tone::Plain)
            }
            field => {
                let shown = display(&settings, field);
                let tone = if shown == "Off" {
                    Tone::Muted
                } else {
                    Tone::Plain
                };
                (shown, tone)
            }
        };
        tone.set_if_neq(wanted);
        text.set_if_neq(Text(shown));
    }
}

/// Registers the settings menu view systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn).add_systems(
        Update,
        (
            show_tab.run_if(resource_changed::<SettingsTab>),
            (show_values, fit_page),
        )
            .run_if(in_state(Menu::Settings)),
    );
}
