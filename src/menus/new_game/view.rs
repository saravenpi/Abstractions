//! Layout of the new world menu.

use bevy::input_focus::AutoFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use super::{Action, Kind, NewWorld};
use crate::menus::slots::Slot;
use crate::menus::{Menu, Nav};
use crate::ui::{
    Cancels, Filled, Icon, Look, Tone, caps, column, entry, footer, heading, hints, item,
    item_with, link, panel, root, text, theme,
};

/// Key hints shown at the bottom of the screen.
const HINTS: [(&str, &str); 3] = [
    ("Switch fields", "Tab or arrows"),
    ("Create", "Enter"),
    ("Back", "Esc"),
];

/// Width of a text field in pixels: wide enough for the longest name and seed.
const FIELD_WIDTH: f32 = theme::MENU_WIDTH * 1.5;

/// Field label with a small gap below it.
fn field_name(content: &str) -> impl Bundle {
    (
        caps(content),
        Node {
            margin: UiRect::bottom(px(theme::GAP_XS)),
            ..default()
        },
    )
}

/// Filled row that holds one text field. It takes focus but does not press like a button.
fn field(kind: Kind, slot: usize) -> impl Bundle {
    (
        Node {
            width: px(FIELD_WIDTH),
            max_width: percent(100),
            height: px(theme::GAP_XL),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(theme::GAP_S)),
            margin: UiRect::bottom(px(theme::GAP_L)),
            overflow: Overflow::clip(),
            ..default()
        },
        Look::default(),
        Filled,
        Hovered::default(),
        BackgroundColor(Color::NONE),
        TabIndex(0),
        Slot(slot),
        kind,
    )
}

/// The typed text of a field.
fn field_text(kind: Kind) -> impl Bundle {
    (
        text("", theme::FONT_BODY, theme::TEXT),
        TextLayout::no_wrap(),
        kind,
    )
}

/// The Create item and Back.
fn actions() -> impl Bundle {
    (
        column(px(theme::MENU_WIDTH), 0.0),
        children![
            (
                item_with(
                    0,
                    entry(Icon::Check, "Create", theme::FONT_BODY, Tone::Accent)
                ),
                Action::Create,
                Slot(2)
            ),
            (
                item(Icon::Back, "Back", Tone::Plain),
                Nav::Back,
                Cancels,
                Slot(3)
            ),
        ],
    )
}

/// The panel with both fields and the actions.
fn card() -> impl Bundle {
    (
        panel(),
        children![
            heading("New world"),
            field_name("Name"),
            (
                field(Kind::Name, 0),
                AutoFocus,
                children![field_text(Kind::Name)]
            ),
            field_name("Seed"),
            (
                field(Kind::Seed, 1),
                children![
                    field_text(Kind::Seed),
                    (
                        link(0, Icon::Shuffle, "Random", Tone::Muted),
                        Action::Random
                    )
                ]
            ),
            actions(),
        ],
    )
}

/// Spawns the menu and resets the typed text.
pub(super) fn spawn(mut commands: Commands, mut input: ResMut<NewWorld>) {
    *input = NewWorld::default();
    commands.spawn((
        DespawnOnExit(Menu::NewGame),
        root(),
        children![card(), footer(hints(&HINTS)),],
    ));
}
