//! Durations of every interface animation, in seconds.
//!
//! Rules: ease-out only, no bounce, no overshoot, no looping pulse except the warp spool bar.

/// Fade of hints, banners, tooltips and HUD elements that change context.
pub const FADE: f32 = 0.12;
/// Fade of a panel as it enters.
pub const PANEL_FADE: f32 = 0.16;
/// Distance in pixels a panel slides toward its edge while it enters.
pub const PANEL_SLIDE: f32 = 8.0;
/// Cross-fade of the map picture when the level changes.
pub const LEVEL_FADE: f32 = 0.20;
/// Time the pointer rests on an icon before its tooltip shows.
pub const TOOLTIP_DELAY: f32 = 0.4;

/// Returns `t` eased out: fast at the start, slow at the end. `t` is clamped to 0 to 1.
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}
