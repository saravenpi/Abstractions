//! Tests that the screen fits every window from 960 by 540 to 2560 by 1440 at every interface
//! size: the zones of the page, the hotbar, the fixed parts of each tab and one whole tool card
//! stack inside the height, and the tabs, each pane, the intent tiles, the line on the intent, a
//! card, the hotbar slots and the footer sit inside the width. The lists scroll inside what is left.

use bevy::math::Vec2;

use super::bench::INTENTS;
use super::cards::WORDS;
use super::detail::BAR_LABEL;
use super::glyph::{CARD, ROW};
use super::intent;
use super::keys::{MAKE_LEGEND, PACK_LEGEND};
use super::rows::{FACT, NAME as ROW_NAME, WEAR};
use super::slots::NAME;
use super::view::{DETAIL, PACK};
use crate::settings::InterfaceSize;
use crate::ui::{HINT_GAP, KEY_GAP, PAGE_FOOT, PAGE_HEAD, STAT_METER, theme, ui_scale};

/// Returns the width in pixels of `chars` glyphs of text of `size`.
fn wide(chars: usize, size: f32) -> f32 {
    chars as f32 * size * theme::GLYPH_ASPECT
}

/// Every window and interface size the game supports, as the logical size it lays out at.
fn layouts() -> Vec<Vec2> {
    let windows = [
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(1920.0, 1080.0),
        Vec2::new(2560.0, 1440.0),
    ];
    let sized = |window: Vec2| InterfaceSize::ALL.map(|size| window / ui_scale(window, size));
    windows.into_iter().flat_map(sized).collect()
}

/// Height of the content zone the panes keep once the hotbar is under them.
fn panes(window: Vec2) -> f32 {
    window.y - PAGE_HEAD - PAGE_FOOT - theme::ROW_TALL - theme::S4
}

/// Height of an action bar with its gap above.
const STRIP: f32 = theme::S4 + theme::ROW_HEIGHT;

/// Height of a tool card that is not focused: its padding, its name and its reason.
const RESTING: f32 = 2.0 * theme::S3 + theme::LINE_BODY + theme::LINE_SMALL;

/// On its tab the workshop keeps room for one whole card under its tiles and the line on the
/// intent, above its actions; on its own the pack keeps room for a group title and two rows above
/// its actions.
#[test]
fn the_panes_fit_every_height() {
    for window in layouts() {
        let room = panes(window);
        let tiles = theme::ROW_TALL + theme::S2 + theme::LINE_SMALL + theme::S3;
        assert!(room - tiles - STRIP >= RESTING, "{window}: {room}");
        let list = room - STRIP - theme::LINE_SMALL - theme::S2;
        assert!(
            list >= theme::ICON + theme::TITLE_BELOW + 2.0 * theme::ROW_TALL,
            "{window}"
        );
        let detail = 2.0 * theme::S4
            + CARD
            + 2.0 * theme::S2
            + 2.0 * theme::LINE_SMALL
            + 4.0 * theme::LINE_SMALL;
        assert!(detail <= room, "{window}: detail");
    }
}

/// Returns the width the workshop pane needs: the widest of its four intent tiles side by side,
/// the line on an intent and a tool card with its picture and its words.
fn workshop_needs() -> f32 {
    let tile = |one| {
        let words = wide(intent::name(one).len(), theme::FONT_BODY)
            .max(wide(intent::need(one).len(), theme::FONT_SMALL));
        2.0 * theme::INSET + theme::ICON + theme::ICON_GAP + words
    };
    let tiles = INTENTS.into_iter().map(tile).fold(0.0, f32::max) * 4.0 + 3.0 * theme::S2;
    let line = INTENTS
        .into_iter()
        .map(|one| intent::line(one).len())
        .max()
        .unwrap_or(0);
    let card = 2.0 * theme::S3 + CARD + theme::S3 + WORDS;
    tiles
        .max(theme::INSET + wide(line, theme::FONT_SMALL))
        .max(card)
}

/// Returns the width the footer needs: Close and its key, then the longer legend of a pane.
fn footer_needs() -> f32 {
    let close = 2.0 * theme::INSET + theme::ICON + theme::ICON_GAP + wide(8, theme::FONT_SMALL);
    let hint = |(words, key): &(&str, &str)| wide(words.len() + key.len(), theme::FONT_SMALL);
    let legend = |all: &[(&str, &str)]| {
        let gaps = HINT_GAP * (all.len() - 1) as f32 + KEY_GAP * all.len() as f32;
        all.iter().map(hint).sum::<f32>() + gaps
    };
    let longer = legend(&PACK_LEGEND[..]).max(legend(&MAKE_LEGEND[..]));
    close + KEY_GAP + theme::GUTTER + longer
}

/// Returns the width the heading zone needs: the two tabs at the page title size, the Tab key,
/// then the load at its longest.
fn header_needs() -> f32 {
    let tab = |name: &str| 2.0 * theme::INSET + wide(name.len(), theme::FONT_HEADING);
    let tabs = tab("Pack") + tab("Workshop") + 2.0 * theme::S2 + wide(3, theme::FONT_SMALL);
    let load = "29.7 of 29.7 kg carried, pace 100 %";
    tabs + theme::GUTTER + wide(load.len(), theme::FONT_SMALL)
}

/// The tabs and the load fit across the heading, the workshop and the pack each fit the width on
/// their own tab, and so do the six hotbar slots and the footer.
#[test]
fn the_panes_fit_every_width() {
    let words = wide(1, theme::FONT_SMALL) + wide(NAME, theme::FONT_SMALL);
    let slot = 2.0 * theme::INSET + words + 2.0 * theme::S2 + ROW;
    let row_room = PACK - 2.0 * theme::INSET - ROW - theme::ICON_GAP;
    let load_row = 2.0 * theme::INSET
        + wide(4, theme::FONT_SMALL)
        + theme::span(3)
        + wide(14, theme::FONT_SMALL)
        + 2.0 * theme::S2;
    let detail_row =
        2.0 * theme::S4 + BAR_LABEL + STAT_METER + 2.0 * theme::S2 + wide(9, theme::FONT_SMALL);
    for window in layouts() {
        let inner = window.x - 2.0 * theme::EDGE;
        assert!(header_needs() <= inner, "{window}: heading");
        assert!(workshop_needs() <= inner, "{window}: workshop");
        assert!(PACK + theme::GUTTER + DETAIL <= inner, "{window}: pack");
        assert!(
            wide(ROW_NAME, theme::FONT_BODY) <= row_room,
            "{window}: row name"
        );
        assert!(
            wide(FACT, theme::FONT_SMALL) + theme::S2 + WEAR <= row_room,
            "{window}: row fact"
        );
        assert!(load_row <= PACK, "{window}: load bar");
        assert!(detail_row <= DETAIL, "{window}: detail");
        assert!(6.0 * slot + 5.0 * theme::S2 <= inner, "{window}: slots");
        assert!(footer_needs() <= inner, "{window}: footer");
    }
}
