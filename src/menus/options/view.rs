//! Layout and live values of the settings menu.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use super::actions::SettingsAction;
use super::binds::conflicted;
use super::field::Field;
use super::format::display;
use super::names::{key_label, key_name};
use super::pages::{self, Page, display_for};
use super::rebind::Rebinding;
use super::rows::Value;
use super::{SettingsTab, Tab, hint, reference};
use crate::menus::{Menu, Nav};
use crate::settings::{Labels, Persistent, Settings};
use crate::ui::{
    Cancels, Scroller, Selected, TipText, Tone, back, entry, hints, item_with, legend, page,
    page_body, page_foot, page_head, page_title, scroll_area, theme,
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
/// describes the focused row. It is [`ROWS`] columns wide at most and narrower when the window
/// has less room, so it never reaches past the window.
fn sheet(active: Tab) -> impl Bundle {
    let node = Node {
        width: px(theme::span(ROWS)),
        min_width: px(0),
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

/// Writes the current setting values into their texts. A key is named as the layout of the
/// player prints it, a value that is off is dimmed, and a key in conflict with another is
/// followed by an exclamation mark. Does nothing while nothing changed.
fn show_values(
    (settings, labels): (Res<Persistent<Settings>>, Labels),
    rebinding: Res<Rebinding>,
    fresh: Query<(), Added<Value>>,
    mut values: Query<(&Value, &mut Text, &mut Tone)>,
) {
    let stale = settings.is_changed() || labels.is_changed() || rebinding.is_changed();
    if !(stale || !fresh.is_empty()) {
        return;
    }
    let (keys, labels) = (&settings.controls.keys, labels.get());
    for (value, mut text, mut tone) in &mut values {
        let (shown, wanted) = match value.0 {
            Field::Key(bind) if rebinding.bind() == Some(bind) => {
                ("Press a key".to_string(), Tone::Plain)
            }
            Field::Key(bind) if conflicted(keys, bind) => (
                format!("{} !", display(&settings, &labels, value.0)),
                Tone::Plain,
            ),
            field => {
                let shown = display(&settings, &labels, field);
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

/// Names the key of each key row in full in its tooltip while the row shows it cut short, and
/// gives the other key rows no tooltip. Does nothing while nothing changed.
fn tip_keys(settings: Res<Persistent<Settings>>, mut rows: Query<(&SettingsAction, &mut TipText)>) {
    for (action, mut tip) in &mut rows {
        let SettingsAction::Rebind(bind) = action else {
            continue;
        };
        if !(settings.is_changed() || tip.is_added()) {
            continue;
        }
        let key = bind.key(&settings.controls.keys);
        let full = key_label(key);
        let wanted = match key_name(key) == full {
            true => String::new(),
            false => full,
        };
        tip.set_if_neq(TipText(wanted));
    }
}

/// Registers the settings menu view systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn).add_systems(
        Update,
        (
            show_tab.run_if(resource_changed::<SettingsTab>),
            show_values,
            tip_keys,
        )
            .run_if(in_state(Menu::Settings)),
    );
}
