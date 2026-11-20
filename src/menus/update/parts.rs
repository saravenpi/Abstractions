//! Building blocks of the update menu: texts, buttons, the notes column and the progress bar.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use super::UpdateAction;
use crate::menus::Nav;
use crate::ui::{Cancels, Icon, Scroller, Tone, item, notice, text, theme};

/// Width of the update menu's content.
pub const WIDTH: f32 = theme::MENU_WIDTH * 2.0;
/// Height of the release notes before they scroll.
pub const NOTES_HEIGHT: f32 = theme::ROW_HEIGHT * 5.0;
/// Thickness of the progress bar.
const BAR: f32 = theme::GAP_XS;

/// Marks the white part of the progress bar.
#[derive(Component, Debug, Clone, Copy)]
pub struct Fill;

/// Marks the line under the progress bar.
#[derive(Component, Debug, Clone, Copy)]
pub struct ProgressText;

/// Marks the line of a page that says why a program outside the game could not be started.
#[derive(Component, Debug, Clone, Copy)]
pub struct Notice;

/// The large line that names a page.
pub fn title(content: impl Into<String>) -> impl Bundle {
    text(content, theme::FONT_HEADING, theme::TEXT)
}

/// A paragraph of dimmed body text, which wraps inside the page.
pub fn para(content: impl Into<String>) -> impl Bundle {
    text(content, theme::FONT_BODY, theme::TEXT_2)
}

/// The empty line a failed restart or a failed opening writes into.
pub fn failure_line() -> impl Bundle {
    (Notice, notice())
}

/// The thin white progress bar, empty, and the line under it.
pub fn bar() -> impl Bundle {
    let fill = (
        Fill,
        Node {
            width: percent(0),
            height: percent(100),
            ..default()
        },
        BackgroundColor(theme::TEXT),
    );
    let track = (
        Node {
            width: percent(100),
            height: px(BAR),
            ..default()
        },
        BackgroundColor(theme::HAIRLINE),
        children![fill],
    );
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(theme::GAP_S),
            ..default()
        },
        children![track, (ProgressText, notice())],
    )
}

/// The main button of a page: its icon when the pack has a fitting one, its label and what it does.
pub type Main<'a> = (Option<Icon>, &'a str, UpdateAction);

/// Spawns the buttons of a page in a menu-wide column: the main action, which takes the focus,
/// and "Later" when the page can be left for later.
pub fn actions(page: &mut ChildSpawnerCommands, main: Option<Main>, later: bool) {
    let column = Node {
        width: px(theme::MENU_WIDTH),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    page.spawn(column).with_children(|column| {
        if let Some((symbol, name, action)) = main {
            column.spawn((item(symbol, name, Tone::Plain), action, AutoFocus));
        }
        if later {
            let back = (item(Icon::Back, "Later", Tone::Plain), Nav::Back, Cancels);
            let mut back = column.spawn(back);
            if main.is_none() {
                back.insert(AutoFocus);
            }
        }
    });
}

/// Scrolls the release notes with Page Up and Page Down, inside their content.
pub(super) fn page_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut notes: Query<(&mut ScrollPosition, &ComputedNode), With<Scroller>>,
) {
    let by = i32::from(keys.just_pressed(KeyCode::PageDown))
        - i32::from(keys.just_pressed(KeyCode::PageUp));
    if by == 0 {
        return;
    }
    for (mut position, node) in &mut notes {
        let room = (node.content_size.y - node.size().y).max(0.0) * node.inverse_scale_factor;
        let wanted = (position.y + by as f32 * NOTES_HEIGHT * 0.5).clamp(0.0, room);
        if wanted != position.y {
            position.y = wanted;
        }
    }
}
