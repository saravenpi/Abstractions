//! The mouse at the helm: it turns the ship only while the left button is held. With the button
//! up it only looks around, the stick is centred and the hull keeps the attitude it has.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::settings::Controls as Options;

/// The mouse button that hands the mouse to the hull while it is held.
pub const STEER: MouseButton = MouseButton::Left;

/// Stick deflection per mouse pixel at sensitivity 1.0.
pub const STICK_PER_PIXEL: f32 = 0.004;

/// Rate per second at which the stick returns to the centre while the mouse rests.
pub const STICK_RETURN: f32 = 3.0;

/// Returns the mouse stick `dt` seconds later, x to the right and y downward, each from -1 to
/// 1: moved by `nudge` and drifting back to the centre while the button is `held`, centred at
/// once when it is not, so that the mouse then steers nothing.
pub fn stick(stick: Vec2, nudge: Vec2, held: bool, dt: f32) -> Vec2 {
    if !held {
        return Vec2::ZERO;
    }
    (stick * (-STICK_RETURN * dt.max(0.0)).exp() + nudge).clamp_length_max(1.0)
}

/// Returns the turn a mouse stick asks of the hull, as pitch up, yaw left and roll left: the
/// nose follows the mouse, up and down and to the sides, and nothing rolls.
pub fn deflection(stick: Vec2) -> DVec3 {
    DVec3::new(f64::from(-stick.y), f64::from(-stick.x), 0.0)
}

/// The mouse as the helm reads it.
#[derive(SystemParam)]
pub struct Mouse<'w> {
    /// Mouse buttons.
    buttons: Res<'w, ButtonInput<MouseButton>>,
    /// Mouse motion of the frame, in pixels.
    motion: Res<'w, AccumulatedMouseMotion>,
}

impl Mouse<'_> {
    /// Returns the mouse stick after this frame of `dt` seconds, with the mouse options of
    /// `controls` applied.
    pub fn stick(&self, before: Vec2, controls: &Options, dt: f32) -> Vec2 {
        let flip = if controls.invert_y { -1.0 } else { 1.0 };
        let scale = STICK_PER_PIXEL * controls.mouse_sensitivity;
        let nudge = self.motion.delta * Vec2::new(scale, scale * flip);
        stick(before, nudge, self.buttons.pressed(STEER), dt)
    }
}
