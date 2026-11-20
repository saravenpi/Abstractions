//! Tuning constants of a touchdown: the landing roll, the slide and a hit on a wall.

/// Speed lost into a surface above which a touchdown may be a landing roll, in metres per second.
pub const ROLL_SINK: f32 = 9.0;

/// Share of the speed that must go into the ground for a landing roll (sine of 30 degrees).
pub const ROLL_SINE: f32 = 0.5;

/// Seconds a landing roll lasts.
pub const ROLL_TIME: f32 = 0.7;

/// Rate at which a landing roll sheds speed, per second.
pub const ROLL_DAMP: f32 = 5.0;

/// Ground speed above which a touchdown becomes a slide, in metres per second.
pub const SLIDE_ENTER: f32 = 9.5;

/// Ground speed at which a slide hands over to the run, in metres per second.
pub const SLIDE_EXIT: f32 = 8.0;

/// Friction of a slide as a share of the weight.
pub const SLIDE_G: f32 = 0.35;

/// Extra slide deceleration per metre per second of speed, per second.
pub const SLIDE_DRAG: f32 = 0.5;

/// Extra share of the speed a wall or cliff hit costs per unit of impact sine.
pub const SCRUB: f32 = 0.3;
