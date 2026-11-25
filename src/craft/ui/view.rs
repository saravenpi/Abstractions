//! The skeleton of the open screen over the blurred game, on the shared page: two tabs, Pack and
//! Workshop, and the load in the heading zone; the pane of the open tab in the content zone, the
//! hotbar under it on both tabs; Close and the key legend in the footer. `paint` fills the holders
//! and shows the pane of the open tab only, so crafting shows only when it is asked for.
//!
//! ```text
//! [ Pack ]  Workshop  Tab                             4.2 of 29.7 kg carried, pace 100 %
//!
//! [~] STONE                          (Workshop tab: [ Cut ][ Pierce ][ Crush ][ Throw ]
//! [ flint                     ]       Slash with a hard edge of flint, obsidian or bone.
//! [ 1.50 kg, stone            ]       [ icon | Flint knife .. Lightest | Reach ### ]
//! Hold Enter   Drop X                 Make Enter   Other head H   Other shaft J   Treat T)
//! [ 1 Flint knife ][ 2 Empty ][ 3 Empty ][ 4 Empty ][ 5 Empty ][ 6 Empty ]
//! Close Esc      Workshop Tab   Choose Up Down   Slot 1-6   Half Shift X
//! ```

use bevy::prelude::*;

use super::Workshop;
use super::bench::Pane;
use super::click::Press;
use crate::ui::{
    back, page, page_body, page_foot, page_head, page_status, scroll_area, strip, text, theme,
};

/// Widest the pack list grows, in pixels: six columns of the grid, so its rows stay readable.
pub const PACK: f32 = theme::span(6);

/// Width of the pack detail inspector beside the list, in pixels: five columns of the grid.
pub const DETAIL: f32 = theme::span(5);

/// Marks the pane of a tab: only the pane of the open tab shows.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tabbed(pub Pane);

/// Marks widgets only visible when the Tools category tab is active in the Workshop.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolsOnly;

/// The holders `paint` fills again when what they show changes.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// The two tabs in the heading zone.
    Tabs,
    /// The carry-load summary bar above the pack list.
    Load,
    /// The 6x4 pack slot grid.
    List,
    /// The details of the focused pack slot beside the grid.
    Detail,
    /// The actions under the pack.
    PackDeeds,
    /// The six crafting category tabs in the Workshop.
    Categories,
    /// The four intent tiles (Tools category).
    Intents,
    /// The tool cards or crafting catalog panels.
    Cards,
    /// The actions under the cards/recipes.
    Deeds,
    /// The six hotbar slots.
    Slots,
    /// The key legend of the pane the keys drive.
    Legend,
}

/// The texts `paint` writes again.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Words {
    /// The load, at the right of the heading zone.
    Status,
    /// The line that explains the chosen intent.
    Explain,
}

/// Returns the left column of the pack pane: the carry-load bar, the rows in a scroll area and
/// the actions under them.
fn pack_list() -> impl Bundle {
    let node = Node {
        width: px(PACK),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        min_height: px(0),
        ..default()
    };
    let load = Node {
        width: percent(100),
        height: px(theme::LINE_SMALL),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(theme::S2),
        padding: UiRect::horizontal(px(theme::INSET)),
        margin: UiRect::bottom(px(theme::S2)),
        ..default()
    };
    (
        node,
        children![
            (Holder::Load, load),
            scroll_area(Holder::List),
            (Holder::PackDeeds, strip()),
        ],
    )
}

/// Returns the pack pane, the Pack tab: the pack list on the left and the focused item's details
/// beside it.
fn pack_pane() -> impl Bundle {
    let node = Node {
        width: percent(100),
        max_width: px(PACK + theme::GUTTER + DETAIL),
        flex_direction: FlexDirection::Row,
        column_gap: px(theme::GUTTER),
        min_height: px(0),
        ..default()
    };
    let detail = Node {
        width: px(DETAIL),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        min_height: px(0),
        ..default()
    };
    (
        Tabbed(Pane::Pack),
        node,
        children![pack_list(), (Holder::Detail, detail)],
    )
}

/// Returns the row of the four intent tiles, at the top of the procedural Tools view.
fn intent_row() -> impl Bundle {
    let node = Node {
        height: px(theme::ROW_TALL),
        flex_shrink: 0.0,
        column_gap: px(theme::S2),
        ..default()
    };
    (Holder::Intents, ToolsOnly, node)
}

/// Returns the line that explains the chosen intent, on the label line of the tiles.
fn explain_line() -> impl Bundle {
    let node = Node {
        min_height: px(theme::LINE_SMALL),
        margin: UiRect::new(px(theme::INSET), px(0), px(theme::S2), px(0)),
        flex_shrink: 0.0,
        ..default()
    };
    (
        Words::Explain,
        ToolsOnly,
        text("", theme::FONT_SMALL, theme::TEXT_2),
        TextLayout::no_wrap(),
        node,
    )
}

/// Returns the room of the tool cards: it takes the height the pane has left, and scrolls.
fn cards_area() -> impl Bundle {
    let node = Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        min_height: px(0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::top(px(theme::S3)),
        ..default()
    };
    (node, children![scroll_area(Holder::Cards)])
}

/// Returns the workshop pane, the Workshop tab: the intent tiles, the line on the chosen intent,
/// the tool cards in a scroll area and the actions under them. It shows only on its tab.
fn workshop_pane() -> impl Bundle {
    let node = Node {
        display: Display::None,
        flex_grow: 1.0,
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (
        Tabbed(Pane::Workshop),
        node,
        children![
            (Holder::Categories, Node::default()),
            intent_row(),
            explain_line(),
            cards_area(),
            (Holder::Deeds, strip()),
        ],
    )
}

/// Returns the content zone: the pane of the open tab, the hotbar under it on both tabs.
fn body() -> impl Bundle {
    let panes = Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        min_height: px(0),
        ..default()
    };
    let slots = Node {
        height: px(theme::ROW_TALL),
        flex_shrink: 0.0,
        column_gap: px(theme::S2),
        margin: UiRect::top(px(theme::S4)),
        ..default()
    };
    (
        page_body(FlexDirection::Column),
        children![
            (panes, children![pack_pane(), workshop_pane()]),
            (Holder::Slots, slots)
        ],
    )
}

/// Spawns the empty screen.
pub fn spawn(commands: &mut Commands) {
    let tabs = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        ..default()
    };
    let head = page_head(children![
        (Holder::Tabs, tabs),
        (Words::Status, page_status(""))
    ]);
    let foot = page_foot(children![
        (Press::Close, back("Close")),
        (Holder::Legend, Node::default())
    ]);
    commands.spawn((DespawnOnExit(Workshop::Open), page(head, body(), foot)));
}
