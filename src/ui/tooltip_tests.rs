//! Tests of where the tooltip goes: under its control, and never past an edge of the window.

use bevy::prelude::*;

use super::{Tip, TipText, place, said};
use crate::ui::theme;

/// A window of the smallest size the game allows.
const SCREEN: Vec2 = Vec2::new(960.0, 540.0);

/// A tooltip 200 by 32 pixels.
const BUBBLE: Vec2 = Vec2::new(200.0, 32.0);

/// In the middle of the window the tooltip is centred under its control, a step below it.
#[test]
fn a_tooltip_sits_centred_under_its_control() {
    let control = Rect::new(400.0, 100.0, 440.0, 124.0);
    assert_eq!(place(control, BUBBLE, SCREEN), Vec2::new(320.0, 128.0));
}

/// At a side edge the tooltip moves in, and under a control at the bottom it goes above it, so
/// all of it stays inside the window.
#[test]
fn a_tooltip_stays_inside_the_window() {
    let corners = [
        Rect::new(0.0, 0.0, 24.0, 24.0),
        Rect::new(936.0, 0.0, 960.0, 24.0),
        Rect::new(936.0, 516.0, 960.0, 540.0),
        Rect::new(0.0, 516.0, 24.0, 540.0),
    ];
    for control in corners {
        let at = place(control, BUBBLE, SCREEN);
        assert!(
            at.x >= theme::S2 && at.x + BUBBLE.x <= SCREEN.x - theme::S2,
            "{at}"
        );
        assert!(at.y >= 0.0 && at.y + BUBBLE.y <= SCREEN.y, "{at}");
    }
    let bottom = Rect::new(400.0, 516.0, 440.0, 540.0);
    assert_eq!(
        place(bottom, BUBBLE, SCREEN).y,
        516.0 - theme::S1 - BUBBLE.y
    );
}

/// A fixed tip wins over changing words, and empty words mean no tooltip.
#[test]
fn a_fixed_tip_wins_and_empty_words_show_nothing() {
    let (tip, text) = (Tip("Fly home"), TipText("Num MemorySubtract".into()));
    assert_eq!(said(Some(&tip), Some(&text)), "Fly home");
    assert_eq!(said(None, Some(&text)), "Num MemorySubtract");
    assert_eq!(said(None, Some(&TipText::default())), "");
}
