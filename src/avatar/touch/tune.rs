//! Tuning constants of a touchdown taken by the whole body: the hard crouch, the roll, the tumble
//! and the skid.

/// Ground speed in meters per second under which a landing roll of the gear is taken as a hard
/// crouch on the spot instead: there is no way to roll along.
pub const ROLL_AHEAD: f32 = 5.0;

/// Speed in meters per second lost into the ground above which a landing roll turns into a
/// tumble, and the speed some way under it where the limbs start to flail.
pub const TUMBLE_LOST: (f32, f32) = (18.0, 24.0);

/// Share of the way from the tuck of a roll to the flung-out limbs a tumbling body flails: a
/// body flung straight out would bounce the pelvis up and down as it turns.
pub const FLAIL: f32 = 0.6;

/// Radius in meters of the body tumbling over the ground: the trunk turns once per this many
/// meters times a full turn, rounded to whole turns.
pub const TUMBLE_RADIUS: f32 = 0.6;

/// Landing hardness, from 0 to 1, at which the first hand and then both hands reach the ground.
pub const HANDS: (f32, f32) = (0.45, 0.8);

/// How much deeper than the crouch a hard landing presses the pelvis down, in meters, and how
/// far the trunk leans forward over the knees, in radians, with both hands on the ground.
pub const HANDS_DOWN: (f32, f32) = (0.23, 0.8);

/// Seconds the landing mode is held at most, on top of the soft hold, after the hardest landing.
pub const LAND_HOLD: f32 = 0.45;

/// Fastest the trunk turns in a roll or a tumble, in radians per second: a roll starts at about
/// this, a tumble is held to it and catches up as the body slows.
pub const SPIN_MAX: f32 = 33.0;

/// Spring rate per second at which a trunk left part way through a turn finishes it.
pub const SETTLE_RATE: f32 = 12.0;

/// Seconds the body takes at least, in the air, to change from coming in upright to coming in
/// flat, or back.
pub const LAIN_SECONDS: f32 = 0.4;

/// Seconds a change into or out of a roll or a skid takes: longer than the long blend, so that
/// going down flat from a glide or rising from the belly never jumps a joint. The turn of a roll
/// does not wait for it.
pub const RISE_SECONDS: f32 = 0.9;
