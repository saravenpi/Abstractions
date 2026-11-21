//! Tests that the screen fits every window from 960 by 540 to 2560 by 1440 at every interface
//! size: the zones of the page, the hotbar, the fixed parts of both panes and one whole tool card
//! stack inside the height, and the panes, the intent tiles, the line on the intent, a card, the
//! hotbar slots and the footer sit inside the width. The lists scroll inside what is left.

use bevy::math::Vec2;

use super::bench::INTENTS;
use super::cards::WORDS;
use super::glyph::{CARD, ROW};
use super::intent;
use super::keys::{MAKE_LEGEND, PACK_LEGEND};
use super::slots::NAME;
use super::view::PACK;
use crate::settings::InterfaceSize;
use crate::ui::{HINT_GAP, KEY_GAP, PAGE_FOOT, PAGE_HEAD, theme, ui_scale};

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

/// The workshop keeps room for one whole card under its title, its tiles and the line on the
/// intent, above its actions; the pack keeps room for a group title and two rows above its
/// actions, which may take two lines on the narrow pane.
#[test]
fn the_panes_fit_every_height() {
    for window in layouts() {
        let room = panes(window);
        let title = theme::ICON + theme::TITLE_BELOW;
        let tiles = theme::ROW_TALL + theme::S2 + theme::LINE_SMALL + theme::S3;
        assert!(room - title - tiles - STRIP >= RESTING, "{window}: {room}");
        let list = room - title - STRIP - theme::ROW_HEIGHT - theme::S1;
        assert!(
            list >= theme::ICON + theme::TITLE_BELOW + 2.0 * theme::ROW_TALL,
            "{window}"
        );
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

/// The pack, the gutter and the workshop fit across; so do the six hotbar slots and the footer.
#[test]
fn the_panes_fit_every_width() {
    let words = wide(1, theme::FONT_SMALL) + wide(NAME, theme::FONT_SMALL);
    let slot = 2.0 * theme::INSET + words + 2.0 * theme::S2 + ROW;
    for window in layouts() {
        let inner = window.x - 2.0 * theme::EDGE;
        let workshop = inner - PACK - theme::GUTTER;
        assert!(workshop_needs() <= workshop, "{window}: {workshop}");
        assert!(6.0 * slot + 5.0 * theme::S2 <= inner, "{window}: slots");
        assert!(footer_needs() <= inner, "{window}: footer");
    }
}
