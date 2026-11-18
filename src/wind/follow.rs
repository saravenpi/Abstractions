//! The wind of the weather: it leans toward the prevailing wind of the climate and gusts in a storm.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Wind;
use crate::space::Anchor;

/// Prevailing wind in metres per second from which the direction follows the climate as far as
/// it ever does.
const STEADY: f32 = 4.0;

/// Most of the direction the prevailing wind takes: the rest still wanders with the clock.
const HOLD: f32 = 0.85;

/// Returns the compass wind `wind`, towards the east and the north at the render origin of
/// `anchor`, on the render XZ plane. It is zero on the flat world and at a pole.
pub fn to_render(anchor: &Anchor, wind: Vec2) -> Vec2 {
    if anchor.flat {
        return Vec2::ZERO;
    }
    let up = anchor.origin.normalize_or_zero();
    let east = DVec3::Y.cross(up).normalize_or_zero();
    let north = up.cross(east);
    let blown = anchor.rotation * (east * f64::from(wind.x) + north * f64::from(wind.y));
    Vec2::new(blown.x as f32, blown.z as f32)
}

/// Returns the clock wind `clock` leaning toward `prevailing`, in render XZ metres per second,
/// and gusting as hard as `storm`, from 0 to 1. With no prevailing wind and no storm it is
/// `clock` itself, bit for bit.
pub fn weathered(clock: Wind, prevailing: Vec2, storm: f32) -> Wind {
    let lean = (prevailing.length() / STEADY).min(1.0) * HOLD;
    let toward = prevailing.normalize_or(clock.direction);
    let direction = match lean > 0.0 {
        true => clock.direction.lerp(toward, lean).normalize_or(toward),
        false => clock.direction,
    };
    let storm = storm.clamp(0.0, 1.0);
    Wind {
        direction,
        strength: clock.strength + (1.0 - clock.strength) * storm,
        ..clock
    }
}
