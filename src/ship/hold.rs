//! Holding the pilot in the seat until the ground beside the hatch is solid.

/// Seconds the pilot is held at the most: past it the pilot steps out on the height of the ground, which
/// the body is put back on when it sinks.
pub const HOLD: f32 = 10.0;

/// Returns the seconds the pilot has been held after a frame of `dt` seconds that began after `held`, or
/// nothing once the pilot may step out: the ground is `solid`, or [`HOLD`] seconds have passed.
pub fn hold(held: f32, solid: bool, dt: f32) -> Option<f32> {
    (!solid && held < HOLD).then_some(held + dt)
}
