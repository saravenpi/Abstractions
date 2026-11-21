//! Tests of contrast over a bright sky, a white one and black: text on `PANEL`, which the menus
//! over the world keep, and HUD text on its shadow, which the in-game HUD has in place of a panel.

use super::color_rig::{holds, on_fill};
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

/// HUD text on the dark pixels of `HUD_SHADOW` beside each stroke. Over a sky of 0.8, the
/// brightest the tonemapper gave in the shots so far, `TEXT` keeps 5.4 to 1, `PIN` 4 to 1 and
/// `ACCENT` 3 to 1; over a pure white sky `TEXT` keeps 3.75 to 1; over black all keep 10 to 1.
/// `TEXT_2` falls under 4.5 to 1 at 0.8, which is why HUD text is `TEXT`.
#[test]
fn hud_text_is_readable_on_its_shadow() {
    let shadow = theme::HUD_SHADOW;
    let floors = [(theme::TEXT, 5.4), (theme::PIN, 4.0), (theme::ACCENT, 3.0)];
    for (text, floor) in floors {
        let got = on_fill(text, shadow, 0.8);
        assert!(got >= floor, "{text:?}: {got} under {floor}");
        assert!(on_fill(text, shadow, 0.0) >= 10.0, "{text:?}");
    }
    assert!(on_fill(theme::TEXT, shadow, 1.0) >= 3.75);
    assert!(on_fill(theme::TEXT_2, shadow, 0.8) < 4.5);
}

/// Warning text on `HUD_SHADOW_WARN`: over a sky of 0.8 `WARN` keeps 4.5 to 1 and `DANGER` 3.5 to
/// 1, over a pure white sky 3.7 and 2.75 to 1, over black 7 to 1. On the plain `HUD_SHADOW`
/// `DANGER` would not reach 3 to 1 at 0.8, which is why warnings cast the deeper shadow.
#[test]
fn warning_text_is_readable_on_its_deeper_shadow() {
    let deep = theme::HUD_SHADOW_WARN;
    for (warning, bright, white) in [(theme::WARN, 4.5, 3.7), (theme::DANGER, 3.5, 2.75)] {
        assert!(on_fill(warning, deep, 0.8) >= bright, "{warning:?}");
        assert!(on_fill(warning, deep, 1.0) >= white, "{warning:?}");
        assert!(on_fill(warning, deep, 0.0) >= 7.0, "{warning:?}");
    }
    assert!(on_fill(theme::DANGER, theme::HUD_SHADOW, 0.8) < 3.0);
}
