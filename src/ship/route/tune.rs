//! Tuning of the climb of a route that leaves a planet.

/// Share, with no unit, of the width of the lock sphere above the surface that a route leaving
/// a planet climbs past before it hands over to the warp drive: the orbit it flies to is that
/// much above the lock, so the drive is free there.
pub const LOCK_MARGIN: f64 = 1.1;
