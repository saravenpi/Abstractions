//! Layout of the new world menu.

use bevy::input_focus::AutoFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use super::{Action, Kind, NewWorld};
use crate::menus::slots::Slot;
use crate::menus::{Menu, Nav};
use crate::ui::{
    Look, Tone, caps, column, footer, hairline, item, item_with, label, link, root, text, theme,
};

/// Section label with a gap below it.
fn heading(content: &str, gap: f32) -> impl Bundle {
    (
        caps(content),
        Node {
            margin: UiRect::bottom(px(gap)),
            ..default()
        },
    )
}

/// Underlined row that holds one text field. It takes focus but does not press like a button.
fn field(kind: Kind, slot: usize, gap: f32) -> impl Bundle {
    (
        Node {
            width: px(theme::MENU_WIDTH * 1.5),
            height: px(theme::SPACE * 5.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::bottom(px(1)),
            margin: UiRect::bottom(px(gap)),
            ..default()
        },
        hairline(),
        Look::default(),
        Hovered::default(),
        TabIndex(0),
        Slot(slot),
        kind,
    )
}

/// The typed text of a field.
fn field_text(kind: Kind) -> impl Bundle {
    (text("", theme::FONT_BODY, theme::TEXT), kind)
}

/// The Create item and Back.
fn actions() -> impl Bundle {
    (
        column(px(theme::MENU_WIDTH), 0.0),
        children![
            (
                item_with(0, label("Create", theme::FONT_BODY, Tone::Accent)),
                Action::Create,
                Slot(2)
            ),
            (item("Back", Tone::Plain), Nav::To(Menu::Main), Slot(3)),
        ],
    )
}

/// Spawns the menu and resets the typed text.
pub(super) fn spawn(mut commands: Commands, mut input: ResMut<NewWorld>) {
    *input = NewWorld::default();
    commands.spawn((
        DespawnOnExit(Menu::NewGame),
        root(),
        children![
            heading("New world", theme::SPACE * 5.0),
            heading("Name", theme::SPACE),
            (
                field(Kind::Name, 0, theme::SPACE * 3.0),
                AutoFocus,
                children![field_text(Kind::Name)]
            ),
            heading("Seed", theme::SPACE),
            (
                field(Kind::Seed, 1, theme::SPACE * 4.0),
                children![
                    field_text(Kind::Seed),
                    (link(0, "Random", Tone::Muted), Action::Random)
                ]
            ),
            actions(),
            footer("Tab or arrows switch fields. Enter creates. Esc goes back"),
        ],
    ));
}
