//! The update entries of the main menu: the quiet line with its "Update" entry, shown once a
//! newer release is known, and the "Check for updates" entry, which is always there.

use bevy::prelude::*;

use super::UpdateAction;
use super::format::{check_label, offer_line};
use crate::menus::{Menu, Nav};
use crate::ui::{Icon, Selected, Tone, icon_slot, item, item_with, label, text, theme};
use crate::update::Updates;

/// Marks the box that holds the line and the "Update" entry. It is hidden until a newer release
/// is known, and hidden entries take no room and no focus.
#[derive(Component, Debug, Clone, Copy)]
pub struct OfferBox;

/// Marks the line that names the newer release.
#[derive(Component, Debug, Clone, Copy)]
pub struct OfferLine;

/// Marks the label of the "Check for updates" entry, which turns into the result of the check.
#[derive(Component, Debug, Clone, Copy)]
pub struct CheckLabel;

/// The line that names the newer release, lined up with the labels of the entries.
fn line() -> impl Bundle {
    (
        OfferLine,
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
        Node {
            padding: UiRect::axes(px(theme::GAP_S), px(theme::GAP_XS)),
            ..default()
        },
    )
}

/// The hidden box with the line and the "Update" entry. The entry is marked as chosen, so it
/// stands out without taking the keyboard focus.
pub fn offer() -> impl Bundle {
    let update = (
        item(Icon::Star, "Update", Tone::Plain),
        Nav::To(Menu::Update),
        Selected(true),
    );
    (
        OfferBox,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            display: Display::None,
            margin: UiRect::bottom(px(theme::GAP_M)),
            overflow: Overflow::clip(),
            ..default()
        },
        children![line(), update],
    )
}

/// The "Check for updates" entry of the main menu.
pub fn check_item() -> impl Bundle {
    let content = (
        Node {
            align_items: AlignItems::Center,
            column_gap: px(theme::GAP_S),
            ..default()
        },
        children![
            icon_slot(Some(Icon::Info), Tone::Plain),
            (
                label("Check for updates", theme::FONT_BODY, Tone::Plain),
                CheckLabel
            ),
        ],
    );
    (item_with(0, content), UpdateAction::Check)
}

/// The texts of the main menu that follow the updater.
type Texts<'w, 's> = (
    Query<'w, 's, &'static mut Text, (With<OfferLine>, Without<CheckLabel>)>,
    Query<'w, 's, &'static mut Text, With<CheckLabel>>,
);

/// Shows the line and the "Update" entry while a newer release is known, and writes the state
/// of the check into its entry. Does nothing while nothing changed.
pub(super) fn show(
    updates: Res<Updates>,
    fresh: Query<(), Added<OfferBox>>,
    mut boxes: Query<&mut Node, With<OfferBox>>,
    (mut lines, mut labels): Texts,
) {
    if !updates.is_changed() && fresh.is_empty() {
        return;
    }
    let release = updates.release();
    let shown = match release {
        Some(_) => Display::Flex,
        None => Display::None,
    };
    for mut node in boxes.iter_mut().filter(|node| node.display != shown) {
        node.display = shown;
    }
    let line = release.map(|release| offer_line(release, updates.stage()));
    for mut text in &mut lines {
        text.set_if_neq(Text(line.clone().unwrap_or_default()));
    }
    for mut text in &mut labels {
        text.set_if_neq(Text(check_label(updates.status())));
    }
}
