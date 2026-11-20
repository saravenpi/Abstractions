//! Map tests: the heading of the favourites holds the button that saves one on one line of the
//! side panel, on every level.

use super::side::SAVE;
use crate::computer::{GLYPH, ROOM};
use crate::ui::theme;

/// The spaced capitals of the heading, a gap and the button, its star, its gap, its words and
/// its padding, fit the room of the panel with either words.
#[test]
fn the_save_button_fits_beside_its_heading() {
    let caps = "Favourites".len() as f32 * (GLYPH + theme::CAPS_SPACING);
    for words in SAVE {
        let button = theme::ICON + theme::GAP_S + words.len() as f32 * GLYPH + theme::GAP_XS;
        assert!(caps + theme::GAP_S + button <= ROOM, "{words}");
    }
}
