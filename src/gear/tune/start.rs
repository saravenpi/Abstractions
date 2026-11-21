//! Tuning constants of the opening of the wings, of the lie of the body, and of the thinnest
//! air a glide holds in.

/// Seconds the wings take to spread or fold: one eased motion, see `FlightState::spread`.
pub const WING_TIME: f32 = 0.4;

/// Natural frequency of the critically damped turn between upright and prone, in radians per
/// second. The body lies into the wing as the wing spreads: it follows the eased spread of
/// [`WING_TIME`] about a third of a second behind and lies nine tenths prone 0.9 s after the
/// press, one motion from the jump or the run to the glide. Faster, a body that lies from
/// upright into a dive would turn more than 7 degrees in a frame.
pub const PRONE_RATE: f32 = 6.0;

/// Fastest the whole body turns, in radians per second (286 degrees per second): the inertia of
/// a body, which no servo of the gear outruns. An opening that lays the body from upright into a
/// dive, the nose that falls at a stall and a pull at the stall angle all turn it at most this
/// fast, under 5 degrees a frame at sixty frames a second.
pub const BODY_RATE: f32 = 5.0;

/// Airspeeds in metres per second between which a body the gear took over turns from upright
/// to prone with the wings folded. Spread, the wings lay it prone at once, see
/// `aim::lain`.
pub const PRONE_SPEED: (f32, f32) = (6.0, 20.0);

/// Fall speeds in metres per second between which a body falling with no gear in use lies
/// from upright into the belly-down arch of a skydiver, the same as `avatar::tune::ARCH_SINK`:
/// a hop never reaches the first, a drop of about 40 m under standard gravity the second. The
/// wings then open from the arch the body is drawn in.
pub const FALL_ARCH: (f32, f32) = (14.0, 28.0);

/// Sink rate in metres per second of the hands-off glide under standard gravity in the home
/// air at sea level: the trim at 4.4 degrees of angle of attack, a lift coefficient of 0.527
/// from [`super::WING_DRAG`] and a path 14 degrees down, so
/// `sqrt(9.81 cos 14 / (AERO 0.527)) sin 14`. The glide sinks at this times
/// `sqrt(g / g0 / bite)`, `bite` being how well the wing bites the air against home.
pub const TRIM_SINK: f32 = 7.25;

/// Most times the home stall speed the stall protection is moved up to in thin air or under a
/// strong pull: past it the air does not carry the wing anyway, see [`MOST_SINK`].
pub const STALL_SCALE: f32 = 4.0;

/// Fastest sink in metres per second the game still calls a glide: the 11 m/s and more of a
/// real suit (https://thaiskyadventures.com/blog/wingsuit-flying/). In air thinner than gives
/// it, the wings spread and the game says the air is too thin to glide, see `carry::glides`.
pub const MOST_SINK: f32 = 11.0;
