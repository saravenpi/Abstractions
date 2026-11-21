//! Tests of the HUD shadow: how far it lies from its glyph in physical pixels at every window,
//! screen density and interface size, and which colour it takes from its text.

use bevy::prelude::*;

use super::shade::{hud_shadow, reach};
use super::{theme, ui_scale};
use crate::settings::InterfaceSize;

/// Returns the physical pixels the shadow of a HUD text lies right of and under its glyph in a
/// window `rows` physical pixels high, 16 by 9, on a screen of `density` physical pixels per
/// logical one, at the interface size `size`.
fn physical(rows: f32, density: f32, size: InterfaceSize) -> Vec2 {
    let logical = Vec2::new(rows * 16.0 / 9.0, rows) / density;
    let scale = ui_scale(logical, size) * density;
    hud_shadow(theme::TEXT, scale).offset * scale
}

/// The owner's limits after the shadow of 2026-11-20 lay too far: one physical pixel at most in
/// a window 720 pixels high, two at most at 1080 and above, at every interface size and on a
/// screen of one or two physical pixels per logical one. It is never less than one, always a
/// whole pixel, and lies as far down as right.
#[test]
fn the_shadow_lies_one_or_two_physical_pixels_away() {
    for size in InterfaceSize::ALL {
        for density in [1.0, 2.0] {
            for rows in [540.0, 720.0, 900.0, 1080.0, 1440.0, 2160.0, 2880.0] {
                let shift = physical(rows, density, size);
                let most = [1.0, 2.0][usize::from(rows > 720.0)];
                assert_eq!(shift.x, shift.y, "{rows} {density} {size:?}");
                assert!((shift.x - shift.x.round()).abs() < 1e-4, "{shift:?}");
                let whole = shift.x.round();
                assert!(
                    (1.0..=most).contains(&whole),
                    "{rows} {density} {size:?}: {whole}"
                );
            }
        }
    }
}

/// The reach is one physical pixel per logical one rounded down, between one and two.
#[test]
fn the_reach_is_rounded_down_and_held() {
    assert_eq!(reach(0.75), 1.0);
    assert_eq!(reach(1.0), 1.0);
    assert_eq!(reach(1.5), 1.0);
    assert_eq!(reach(2.0), 2.0);
    assert_eq!(reach(6.0), theme::HUD_SHADOW_MOST);
    assert_eq!(hud_shadow(theme::TEXT, 0.0).offset, Vec2::ONE);
    assert_eq!(hud_shadow(theme::TEXT, f32::NAN).offset, Vec2::ONE);
}

/// Text and icons in `TEXT`, `TEXT_2`, `PIN` and `ACCENT` cast `HUD_SHADOW`, text in `WARN` and
/// `DANGER` the deeper `HUD_SHADOW_WARN`. A text fading out takes its shadow with it, and a text
/// with no colour casts none.
#[test]
fn the_shadow_takes_its_colour_from_its_text() {
    for tone in [theme::TEXT, theme::TEXT_2, theme::PIN, theme::ACCENT] {
        assert_eq!(hud_shadow(tone, 1.0).color, theme::HUD_SHADOW, "{tone:?}");
    }
    for tone in [theme::WARN, theme::DANGER] {
        assert_eq!(hud_shadow(tone, 1.0).color, theme::HUD_SHADOW_WARN);
    }
    let half = hud_shadow(theme::TEXT.with_alpha(theme::TEXT_2.alpha() * 0.5), 1.0);
    assert!((half.color.alpha() - theme::HUD_SHADOW.alpha() * 0.5).abs() < 1e-6);
    let faded = hud_shadow(theme::WARN.with_alpha(0.0), 1.0).color;
    assert_eq!(faded.alpha(), 0.0);
    assert_eq!(hud_shadow(Color::NONE, 1.0).color.alpha(), 0.0);
    assert!(theme::HUD_SHADOW.alpha() < theme::HUD_SHADOW_WARN.alpha());
}
