//! Tuning constants of the orbital cruise and of the glide that ends it, each with its unit.

use crate::universe::STANDARD_GRAVITY;

/// Radians of the planet the full lever sweeps per second in cruise: two degrees. Half a turn
/// takes 90 s on any planet, 226 km/s over an Earth 111 km up. No thrust holds a circle at that
/// speed (it would take 790 g), so the warp drive carries the ship as it does in warp: no load on
/// the crew and no velocity of its own past what the drive gives it.
pub const RATE: f64 = 0.034_906_585;

/// Radii under the edge of the lock sphere that the highest cruise keeps, so that it never
/// leaves the sphere it runs in.
pub const LOCK_SLACK: f64 = 0.01;

/// Share of the fastest glide that a whole glide averages, for the time to the target the HUD
/// shows: it slows to the drag of the low air at its end.
pub const PACE: f64 = 0.6;

/// Share of its height over the cruise floor that the held height moves by per second at full
/// vertical stick, and the slowest such move in metres per second: from 800 km down to the floor
/// in about ten seconds, and 2 km/s away from it.
pub const CLIMB: (f64, f64) = (0.5, 2000.0);

/// Radians per second the heading of a manual cruise turns at full yaw.
pub const TURN: f64 = 0.5;

/// Seconds in which the nose of a cruising ship closes most of the gap to its track.
pub const ALIGN: f64 = 0.5;

/// Angle in radians below the horizon of the glide into the air: 40 degrees, within the 35 to 45
/// of the orbital glide of Elite Dangerous.
pub const GLIDE_ANGLE: f64 = 0.698_131_7;

/// Hardest slowing in metres per second squared the glide asks while it follows the heating
/// bound down: two gravities. The bound falls as the sixth root of the density, so a glide at
/// `v` along [`GLIDE_ANGLE`] through air of scale height `H` slows at `v^2 sin(angle) / 6H`.
pub const GLIDE_PULL: f64 = 2.0 * STANDARD_GRAVITY;

/// Share of the speed at which the nose would start to glow that the glide flies at: the speed
/// hold lags the falling bound by a fraction of a second.
pub const COOL_SHARE: f64 = 0.9;

/// Slowest glide in metres per second, over a body with no air to set it.
pub const GLIDE_LEAST: f64 = 500.0;

/// Metres above the ground the glide aims for, and the metres along the ground before the target
/// it aims at: the glide ends a few kilometres up and a few short, the target ahead.
pub const END: (f64, f64) = (2000.0, 8000.0);

/// Metres over the aim height in which the glide flattens, and the least share of its angle it
/// keeps there: it ends ten degrees under the horizon, not in a dive.
pub const FLARE: (f64, f64) = (4000.0, 0.25);

/// Metres over the aim height at which the glide hands the ship back to the pilot.
pub const DONE: f64 = 1000.0;

/// Tilt of the pitch stick toward nose up past which the pilot takes the glide back.
pub const ABORT: f64 = 0.5;

/// Seconds a sentence about how the cruise ended stays on the prompt line.
pub const NOTE_TIME: f64 = 6.0;
