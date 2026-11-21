//! Tests of the contrast of text on the HUD fills over a bright sky, a white one and black.

use bevy::prelude::*;

use super::color_rig::{holds, on_fill, on_panel};
use super::theme;

/// Text on the light `PANEL`, over a pure white sky, the worst case no tonemapper reaches: `TEXT`
/// keeps 5.5 to 1, the secondary text 3.5 to 1, and the pin and accent colours 4 and 3 to 1.
#[test]
fn text_on_the_panel_is_readable_over_a_white_sky() {
    let (t, t2, pin, accent) = (theme::TEXT, theme::TEXT_2, theme::PIN, theme::ACCENT);
    holds(1.0, &[(t, 5.5), (t2, 3.5), (pin, 4.0), (accent, 3.0)]);
}

/// Over a bright sky of 0.8, which the game's tonemapper does not exceed in the shots taken so far
/// (an assumption to check on new shots), the secondary text reaches 4.5 to 1. Over black every
/// text keeps 7 to 1 or more.
#[test]
fn text_on_the_panel_is_readable_over_a_bright_sky_and_black() {
    let (t, t2, pin, accent) = (theme::TEXT, theme::TEXT_2, theme::PIN, theme::ACCENT);
    holds(0.8, &[(t2, 4.5), (pin, 5.0), (accent, 4.0)]);
    holds(0.0, &[(t, 15.0), (t2, 7.0), (pin, 7.0), (accent, 7.0)]);
}

/// The rows that can hold warning text sit on `PANEL_DEEP`, where `WARN` and `DANGER` keep 4.5 to 1
/// over a white sky and 7 to 1 over black. They do not reach 3 to 1 on the light `PANEL`, which is
/// why those rows have the deeper fill.
#[test]
fn warning_text_is_readable_on_the_deep_panel() {
    for warning in [theme::WARN, theme::DANGER] {
        let white = on_fill(warning, theme::PANEL_DEEP, 1.0);
        assert!(white >= 4.5, "{warning:?} {white}");
        assert!(on_fill(warning, theme::PANEL_DEEP, 0.0) >= 7.0);
    }
    assert!(on_panel(theme::DANGER, 1.0) < 3.0);
    assert!(theme::PANEL.alpha() < theme::PANEL_DEEP.alpha());
}
