//! The skeleton of the open screen over the blurred game, on the shared page: the title and the
//! load in the heading zone; the pack on the left and the workshop on the right in the content
//! zone, the hotbar under both; Close and the key legend in the footer. `paint` fills the holders.
//!
//! ```text
//! Pack and workshop                                   4.2 of 29.7 kg carried, pace 100 %
//!
//! [b] PACK                      [w] WORKSHOP
//!     [~] STONE                 [ Cut   ][ Pierce ][ Crush  ][ Throw  ]
//!     [ flint        ]          Slash with a hard edge of flint, obsidian or bone.
//!     [ 1.50 kg      ]          [ picture | Flint knife .. Lightest | Reach ### ]
//!                               [ picture | Bone axe ..... Hardest  | ...       ]
//!                               Make Enter   Other head H   Other shaft J   Treat T
//! [ 1 Flint knife ][ 2 Empty ][ 3 Empty ][ 4 Empty ][ 5 Empty ][ 6 Empty ]
//! Close Esc      Workshop Tab   Choose Up Down   Slot 1-6   Half Shift X
//! ```

use bevy::prelude::*;

use super::Workshop;
use super::click::Press;
use crate::ui::{
    Icon, back, page, page_body, page_foot, page_head, page_status, page_title, scroll_area,
    section_title, strip, text, theme,
};

/// Width in pixels of the pack pane: four columns of the grid.
pub const PACK: f32 = theme::span(4);

/// The holders `paint` fills again when what they show changes.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// The groups of rows of the pack.
    List,
    /// The actions under the pack.
    PackDeeds,
    /// The four intent tiles.
    Intents,
    /// The tool cards, or what is missing.
    Cards,
    /// The actions under the cards.
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

/// Returns the pack pane: its title, then its rows in a scroll area.
fn pack_pane() -> impl Bundle {
    let node = Node {
        width: px(PACK),
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::TITLE_BELOW),
        ..default()
    };
    (
        node,
        children![
            section_title(Icon::Pack, "Pack"),
            scroll_area(Holder::List),
            (Holder::PackDeeds, strip())
        ],
    )
}

/// Returns the row of the four intent tiles, under the title of the workshop.
fn intent_row() -> impl Bundle {
    let node = Node {
        height: px(theme::ROW_TALL),
        flex_shrink: 0.0,
        column_gap: px(theme::S2),
        margin: UiRect::top(px(theme::TITLE_BELOW)),
        ..default()
    };
    (Holder::Intents, node)
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

/// Returns the workshop pane: its title, the intent tiles, the line on the chosen intent, the tool
/// cards in a scroll area and the actions under them.
fn workshop_pane() -> impl Bundle {
    let node = Node {
        flex_grow: 1.0,
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    (
        node,
        children![
            section_title(Icon::Workshop, "Workshop"),
            intent_row(),
            explain_line(),
            cards_area(),
            (Holder::Deeds, strip()),
        ],
    )
}

/// Returns the content zone: the two panes side by side, the hotbar under them.
fn body() -> impl Bundle {
    let panes = Node {
        flex_grow: 1.0,
        flex_basis: px(0),
        min_height: px(0),
        column_gap: px(theme::GUTTER),
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
    let head = page_head(children![
        page_title("Pack and workshop"),
        (Words::Status, page_status(""))
    ]);
    let foot = page_foot(children![
        (Press::Close, back("Close")),
        (Holder::Legend, Node::default())
    ]);
    commands.spawn((DespawnOnExit(Workshop::Open), page(head, body(), foot)));
}
