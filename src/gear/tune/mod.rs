//! Tuning constants of the flight gear. Speeds are quoted under standard gravity at sea level.

use bevy::prelude::*;

/// The constants of the opening of the wings and of the lie of the body.
mod start;
/// The constants of a touchdown.
mod touch;
/// The constants of the boot water jet.
mod water;

pub use {start::*, touch::*, water::*};

/// Air density at sea level in kilograms per cubic metre, the same as the ship uses.
pub const SEA_DENSITY: f32 = 1.2;

/// Gravity in metres per second squared the quoted speeds are measured under: standard gravity.
pub const REF_GRAVITY: f32 = crate::universe::STANDARD_GRAVITY as f32;

/// Longest internal integration step in seconds. Longer frames are cut into equal substeps.
pub const MAX_STEP: f32 = 1.0 / 120.0;

/// Most substeps one call may take.
pub const MAX_SUBSTEPS: f32 = 8.0;

/// Drag area over twice the mass at sea level density, per metre: the 0.0453 tuned under 22 m/s
/// squared times 9.80665 over 22, so that every speed of the gear stays the same.
pub const AERO: f32 = 0.020_193;

/// Drag coefficients of the spread wingsuit along the right, dorsal and nose axes.
///
/// The dorsal one is the wing itself: it makes the lift. On a flow at angle of attack `a` they
/// give a lift coefficient `(dorsal - nose) * sin(a) * cos(a)` and a drag coefficient
/// `nose * cos(a)^2 + dorsal * sin(a)^2`, so the best glide ratio is
/// `(dorsal - nose) / (2 * sqrt(dorsal * nose))`, 4.3 here, at about 7 degrees.
///
/// Real suits glide at about 2.5 to 3, forward at up to 28 m/s and down at 11 m/s and more
/// (https://www.jafmonline.net/article_2307_6211080fa89981f66b1a0c9d55c61d0f.pdf,
/// https://thaiskyadventures.com/blog/wingsuit-flying/). The game is kinder: under
/// [`REF_GRAVITY`] a glide of four to one flies at about 30 m/s and sinks at about 7, the wing
/// stalls under about 15 m/s at [`AOA_MAX`], and a dive runs past 50 m/s.
pub const WING_DRAG: Vec3 = Vec3::new(0.6, 6.95, 0.091);

/// Drag coefficient of the folded body standing upright or diving head first, the same on
/// every axis: a terminal speed of about 118 m/s under [`REF_GRAVITY`].
pub const BODY_DRAG: f32 = 0.035;

/// Drag coefficient of the folded body lying flat across its fall, belly to the air, the same
/// on every axis: a terminal speed of about 82 m/s under [`REF_GRAVITY`], which is the 55 m/s
/// of a skydiver belly to earth (https://en.wikipedia.org/wiki/Terminal_velocity) scaled by the
/// root of the gravity ratio. [`BODY_DRAG`] scales the same way to about 80 m/s head down; the
/// same page gives about 90 m/s with the limbs pulled in.
pub const BELLY_DRAG: f32 = 0.072;

/// Density ratio to sea level at which the gear still has 63 percent of its grip on the air.
///
/// It sets the ceiling of powered flight: see [`CEILING_SHARE`].
pub const THIN_AIR: f32 = 0.15;

/// Height above the sea, as a share of the atmosphere height, up to which powered flight still
/// holds its height: about 3 scale heights, where 5 percent of the air is left. That is 26 km
/// on a planet with the 8.4 km scale height of the Earth.
///
/// It is not a limit applied anywhere: it is what [`THIN_AIR`], the wing and the thrust give,
/// written down and held by a test. To let the wingsuit climb higher, lower [`THIN_AIR`] and
/// raise this number with it.
pub const CEILING_SHARE: f32 = 0.253;

/// Booster thrust in powered flight, as a share of the local weight.
pub const THRUST_G: f32 = 0.9;

/// Largest vertical force the boosters top up to at rest, as a share of the weight. Well below
/// 1, so that the body can never hover or rise on the boosters alone.
pub const SUPPORT_G: f32 = 0.8;

/// Extra share of the weight the top-up reaches once the body runs forward through the air.
/// With [`SUPPORT_G`] it stays below 1.
pub const SUPPORT_RUN: f32 = 0.16;

/// Ground speeds between which the forward run earns [`SUPPORT_RUN`], in metres per second.
pub const SUPPORT_RUN_SPEED: (f32, f32) = (4.0, 16.0);

/// Airspeeds between which the low speed support fades out, in metres per second.
pub const SUPPORT_FADE: (f32, f32) = (24.0, 38.0);

/// Share of the angle between the nose and the look direction the thrust turns by.
pub const VECTORING: f32 = 0.35;

/// Largest tangent of the thrust vectoring angle (about 20 degrees).
pub const VECTOR_LIMIT: f32 = 0.36;

/// Descent speed the boots brake a fall to under [`REF_GRAVITY`], in metres per second.
pub const BOOT_DESCENT: f32 = 5.0;

/// Slowest and fastest braked descent on any planet, in metres per second.
pub const BOOT_DESCENT_RANGE: (f32, f32) = (4.0, 6.0);

/// Largest upward push of the braking boots, as a multiple of the weight.
pub const BRAKE_MAX_G: f32 = 4.5;

/// Horizontal acceleration the braking boots give, in metres per second squared.
pub const BOOT_AIR_ACCEL: f32 = 6.0;

/// Horizontal damping of the braking boots per second. Their top drift is accel over damping.
pub const BOOT_AIR_DAMP: f32 = 0.6;

/// Horizontal acceleration of a free falling body leaning into the air, at sea level.
pub const FALL_AIR_ACCEL: f32 = 2.0;

/// Seconds the boosters take to spool up or down.
pub const THRUST_TIME: f32 = 0.25;

/// Natural frequency of the critically damped pitch response, in radians per second.
pub const PITCH_RATE: f32 = 9.0;

/// Natural frequency of the critically damped pull, in radians per second: the angle of
/// attack answers faster than the flight frame turns, as a wing does, and a held key reaches
/// the load it asks for in about a tenth of a second.
pub const PULL_RATE: f32 = 30.0;

/// Natural frequency of the critically damped bank response, in radians per second.
pub const BANK_RATE: f32 = 6.0;

/// Rate at which the nose heading follows its target while the wings are spread, per second.
pub const YAW_RATE: f32 = 12.0;

/// Steepest a folded body or a view that steers may point the flight frame above or below the
/// horizon, in radians (80 degrees). On the wings the keys know no such limit.
pub const MAX_PITCH: f32 = 1.4;

/// Rate in radians per second at which a held roll key rolls the wings: a whole roll in a
/// little over two seconds. Held on, the body rolls on round, through inverted flight.
pub const ROLL_RATE: f32 = 3.0;

/// Rate in radians per second at which a held back key turns the flight path toward the back
/// of the body, as far as the wing may pull: a loop in a few seconds, given the speed. A glide
/// needs about 70 m/s for a loop, about the fastest dive, so loops want the boosters.
pub const PULL_TURN: f32 = 2.0;

/// Share of the self-level the flight path angle gets while the wing flies clear of the stall,
/// from 0 to 1: let go of the pitch keys, the path comes back toward the best glide in about
/// five seconds and not in one, so the wings fly on where the pilot pointed them and the roll
/// alone levels at [`LEVEL`]. At the stall the nose still comes down at the full rate.
///
/// A self-level that also pitches fights the pilot; wingsuit and flight games hold the
/// attitude and level the roll only (https://vazgriz.com/346/).
pub const LEVEL_PITCH: f32 = 0.2;

/// Strength of the self-level, from 0 to 1. Let go of the keys, the wings come level and the
/// nose comes back toward the best glide: at 1 in about a second, at 0 not at all, the body
/// then holding the attitude the keys left. Folded, the body always rights itself in full.
pub const LEVEL: f32 = 0.8;

/// Steepest bank the view may ask for while it steers, in radians (55 degrees).
pub const BANK_LIMIT: f32 = 0.96;

/// Lowest flight path angle the view may ask for while it steers, in radians (70 degrees down).
pub const DIVE_LIMIT: f32 = -1.22;

/// Highest flight path angle the view may ask for while it steers, in radians (30 degrees up).
pub const CLIMB_LIMIT: f32 = 0.52;

/// Rate per second at which the asked flight path angle follows the view: 95 percent of a
/// change in 0.6 s. Let go of the keys it follows the best glide at [`LEVEL`] of this rate.
pub const COMMAND_RATE: f32 = 5.0;

/// Rate in radians per second at which a full yaw key turns the flight path flat.
pub const YAW_TURN: f32 = 0.45;

/// Bank angle per radian of heading error between the view and the flight path, while the
/// view steers.
pub const BANK_GAIN: f32 = 1.6;

/// Airspeeds between which the stall protection lets go: under the first a hands-off wing
/// holds its nose down at [`STALL_PITCH`], and a view that steers banks no more than
/// [`STALL_BANK`] of its limit. A held key may still stall the wing.
pub const STALL_SPEED: (f32, f32) = (14.0, 21.0);

/// Highest flight path angle the wings may be asked for at the stall, in radians.
pub const STALL_PITCH: f32 = -0.35;

/// Share of [`BANK_LIMIT`] left at the stall.
pub const STALL_BANK: f32 = 0.35;

/// Gap in radians between the asked flight path angle and a lower path over which a hands-off
/// wing clear of the stall pulls out of its dive at full load, not at the best glide angle.
pub const PULL_OUT: f32 = 0.3;

/// Largest angle of attack the nose may hold above the flight path, in radians (18 degrees).
pub const AOA_MAX: f32 = 0.314;

/// Largest angle the nose may hold below the flight path, in radians.
pub const AOA_MIN: f32 = 0.17;

/// Largest load in g the wing may pull, a held key included.
pub const MAX_LOAD: f32 = 4.0;

/// Largest nose-up trim that holds the flight path on the look direction, in radians.
pub const TRIM_MAX: f32 = 0.2;

/// Time constant in seconds with which the wings turn the flight path onto the asked angle: the
/// path rate is the gap over this time, within [`MAX_LOAD`].
///
/// A path that lags the attitude by about 0.3 s reads crisp at cruise speed
/// (https://oucampustest.sjsu.edu/researchfoundation/docs/VFS_2021_Berger.pdf). A load per
/// radian instead gave a lag that grew with the speed, 0.34 s at 30 m/s and 0.8 s at 70.
pub const PATH_TIME: f32 = 0.3;

/// Fastest a banked pull turns the heading, in radians per second (40 degrees per second): a
/// full circle in about nine seconds at the glide speed.
///
/// Wingsuits drift through turns rather than carve them
/// (https://www.skydivemag.com/new/thoughts-on-terrain-flying/), and a glider at 30 m/s turns
/// 19 to 32 degrees per second at 45 to 60 degrees of bank under Earth gravity
/// (https://www.gliding.co.uk/wp-content/uploads/sites/3/2022/06/14-ADVANCED-TURNING-1st-Review.pdf).
/// Only the share of the lift that turns the heading is held to it, so a loop or a pull-up
/// with the wings level keeps the whole of [`MAX_LOAD`] (https://vazgriz.com/346/).
pub const TURN_CAP: f32 = 0.7;

/// Largest share of the lift in g that may turn the heading: at speed it holds the turn rate
/// under [`TURN_CAP`], falling as one over the speed, 18 degrees per second at 70 m/s.
pub const TURN_LOAD: f32 = 2.2;

/// Largest load factor the trim compensates for in a banked turn.
pub const TRIM_LOAD: f32 = 2.5;

/// Horizontal speeds between which the nose heading leaves the look direction for the flight path.
pub const FOLLOW_SPEED: (f32, f32) = (4.0, 14.0);

/// Airspeeds between which the wings start to bank.
pub const BANK_SPEED: (f32, f32) = (8.0, 20.0);

/// Airspeeds between which the pitch limits around the flight path come in.
pub const LIMIT_SPEED: (f32, f32) = (3.0, 12.0);

/// Seconds to impact between which the ground assist goes from full to none.
pub const FLARE_TIME: (f32, f32) = (0.6, 2.0);

/// Sines of the descent angle between which the ground assist comes in.
pub const FLARE_STEEP: (f32, f32) = (0.2, 0.6);

/// Largest nose-up bias of the ground assist, in radians.
pub const FLARE_PITCH: f32 = 0.45;
