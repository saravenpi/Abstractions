//! Layout and live values of the settings menu.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

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
    Cancels, Scroller, Selected, Tone, back, entry, hints, item_with, legend, page, page_body,
    page_foot, page_head, page_title, scroll_area, theme,
};

/// Key hints shown in the footer, after the back action.
const HINTS: [(&str, &str); 3] = [
    ("Move", "Up Down"),
    ("Change a value", "Left Right"),
    ("Rebind a key", "Enter"),
];

/// Columns of the alignment grid the tabs take.
pub(super) const TABS: u32 = 3;

/// Columns of the alignment grid the page of a tab takes.
pub(super) const ROWS: u32 = 8;

/// One tab item in the left column. The open tab is selected, which is not the keyboard focus.
fn tab_item(tab: Tab, active: Tab) -> impl Bundle {
    let (symbol, name) = tab.face();
    (
        item_with(0, entry(symbol, name, theme::FONT_BODY, Tone::Plain)),
        Selected(tab == active),
        SettingsAction::Tab(tab),
    )
}

/// Left column with the tabs. The open one takes the focus.
fn sidebar(active: Tab) -> impl Bundle {
    let tabs = SpawnWith(move |parent: &mut ChildSpawner| {
        for tab in Tab::ALL {
            let mut item = parent.spawn(tab_item(tab, active));
            if tab == active {
                item.insert(AutoFocus);
            }
        }
    });
    let node = Node {
        width: px(theme::span(TABS)),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (node, Children::spawn(tabs))
}

/// The pages in one scrolling column that takes the height the window has, over the line that
/// describes the focused row.
fn sheet(active: Tab) -> impl Bundle {
    let node = Node {
        width: px(theme::span(ROWS)),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        ..default()
    };
    let all = children![
        pages::graphics(active),
        pages::controls(active),
        pages::ship(active),
        pages::audio(active),
        pages::general(active),
        reference::help(active),
    ];
    (node, children![scroll_area(all), hint::line()])
}

/// Spawns the whole settings menu on the tab used last.
fn spawn(mut commands: Commands, tab: Res<SettingsTab>) {
    let body = (
        page_body(FlexDirection::Row),
        children![sidebar(tab.0), sheet(tab.0)],
    );
    let foot = page_foot(children![
        (back("Back"), Nav::Back, Cancels),
        legend(hints(&HINTS))
    ]);
    commands.spawn((
        DespawnOnExit(Menu::Settings),
        page(page_head(children![page_title("Settings")]), body, foot),
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
            show_values,
        )
            .run_if(in_state(Menu::Settings)),
    );
}
