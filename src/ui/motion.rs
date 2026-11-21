//! Durations of every interface animation, in seconds.
//!
//! Rules: ease-out only, no bounce, no overshoot, no looping pulse except the warp spool bar.
//! The title slideshow loops, but it is a slow drift and a cross-fade, never a pulse.

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
/// Seconds from one picture of the title slideshow to the next, the cross-fade included.
pub const SLIDE: f32 = 12.0;
/// Seconds of the cross-fade between two pictures of the title slideshow, at the end of [`SLIDE`].
pub const SLIDE_FADE: f32 = 1.5;
/// Share of its size a picture of the title slideshow grows or shrinks by while it shows, a
/// ratio: the slow drift that keeps it from ever standing still.
pub const SLIDE_DRIFT: f32 = 0.04;

/// Returns `t` eased out: fast at the start, slow at the end. `t` is clamped to 0 to 1.
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Returns `alpha` moved `dt` seconds toward 1 when `shown`, toward 0 otherwise, in [`FADE`]
/// seconds for the whole way.
pub fn approach(alpha: f32, shown: bool, dt: f32) -> f32 {
    let step = dt / FADE;
    match shown {
        true => (alpha + step).min(1.0),
        false => (alpha - step).max(0.0),
    }
}
