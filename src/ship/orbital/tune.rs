//! Tuning constants of the orbital cruise and of the glide that ends it, each with its unit.

/// Radians of the planet the full lever sweeps per second in cruise: six degrees. Half a turn
/// takes 30 s on any planet, 680 km/s over an Earth 111 km up. No thrust holds a circle at that
/// speed (it would take 7,000 g), so the warp drive carries the ship as it does in warp: no load
/// on the crew and no velocity of its own past what the drive gives it.
pub const RATE: f64 = 0.104_719_755;

/// Seconds in which the speed of the cruise grows by the factor e from the slowest warp speed,
/// and falls by it toward the start of the glide: about four seconds to full speed, and four to
/// slow down.
pub const SPOOL: f64 = 0.75;

/// Radians under the edge of the lock sphere that the highest cruise keeps, so that it never
/// leaves the sphere it runs in.
pub const LOCK_SLACK: f64 = 0.01;

/// Share of its height over the cruise floor that the held height moves by per second at full
/// vertical stick, and the slowest such move in metres per second: from 800 km down to the floor
/// in about ten seconds, and 2 km/s away from it.
pub const CLIMB: (f64, f64) = (0.5, 2000.0);

/// Radians per second the heading of a manual cruise turns at full yaw.
pub const TURN: f64 = 0.5;

/// Fastest turn of the ship onto the course of a cruise, in radians per second (45 degrees a
/// second), and the angular acceleration in radians per second squared it eases in and out with:
/// half a turn in 4.5 s.
pub const SLEW: (f64, f64) = (std::f64::consts::FRAC_PI_4, 1.5);

/// Radians between the nose and the course under which the cruise starts to build its speed:
/// five degrees.
pub const ALIGNED: f64 = 0.087_266_46;

/// Largest stick the glide gives the attitude thrusters: 0.6 of the pitch rate of the assist,
/// 38 degrees a second, so it noses down into the glide as a pilot would.
pub const GLIDE_STICK: f64 = 0.6;

/// Angle in radians below the horizon of the glide into the air: 65 degrees, the steepest that
/// still flattens out over the base. The glide starts 58 km short of the target on an Earth.
pub const GLIDE_ANGLE: f64 = 1.134_464;

/// Fastest glide in metres per second, over a body with no air to set it.
pub const GLIDE_MOST: f64 = 8000.0;

/// Metres above the ground the glide aims for, and the metres along the ground before the target
/// it aims at: the glide ends about 2 km up and a few kilometres short, the target ahead.
pub const END: (f64, f64) = (1500.0, 7000.0);

/// Metres over the aim height in which the glide flattens, and the least share of its angle it
/// keeps there: it ends under ten degrees from the horizon, not in a dive.
pub const FLARE: (f64, f64) = (3000.0, 0.1);

/// Metres over the aim height at which the glide hands the ship back to the pilot.
pub const DONE: f64 = 400.0;

/// Tilt of the pitch stick toward nose up past which the pilot takes the glide back.
pub const ABORT: f64 = 0.5;

/// Seconds a sentence about how the cruise ended stays on the prompt line.
pub const NOTE_TIME: f64 = 6.0;
