//! Tuning of the climb of a route that leaves a planet.

/// Share, with no unit, of the width of the lock sphere above the surface that a route leaving
/// a planet climbs past before it hands over to the warp drive: the orbit it flies to is that
/// much above the lock, so the drive is free there.
pub const LOCK_MARGIN: f64 = 1.1;

/// Largest stick, from 0 to 1, the climb gives the attitude thrusters: 0.6 of the turn rates of
/// the assist, under 40 degrees a second in pitch, so the ship turns into its climb as a pilot
/// would and never swings round in a moment.
pub const CLIMB_STICK: f64 = 0.6;
