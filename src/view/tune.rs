//! Tuning constants of the camera rig. Distances are metres, angles radians, rates per second.

/// Radians of rotation per mouse pixel at sensitivity 1.0, the value the walking camera always had.
pub const RADIANS_PER_PIXEL: f32 = 0.0022;

/// Wheel pixels that count as one zoom notch on devices that scroll by pixels.
pub const PIXELS_PER_NOTCH: f32 = 40.0;

/// Largest look-up or look-down angle (89 degrees).
pub const PITCH_LIMIT: f32 = 1.553_343;

/// Farthest the head of the pilot turns to each side in the seat of the ship (115 degrees).
pub const COCKPIT_TURN: f32 = 2.0;

/// Farthest the head of the pilot looks up in the seat of the ship (52 degrees): beyond it the
/// eyes would swing back into the headrest.
pub const COCKPIT_UP: f32 = 0.9;

/// Largest look-up angle of the third person orbit (70 degrees).
pub const THIRD_UP: f32 = 1.221_73;

/// Largest look-down angle of the third person orbit (80 degrees).
pub const THIRD_DOWN: f32 = 1.396_263;

/// Shortest and longest simulated frame.
pub const DT_RANGE: (f32, f32) = (1.0e-4, 0.1);

/// Seconds a switch between first and third person takes.
pub const SWITCH_TIME: f32 = 0.35;

/// Seconds the blend between the ground rig and the flight rig takes, in both directions.
pub const FLIGHT_TIME: f32 = 0.5;

/// Rate at which the third person orbit catches up with the look angles.
pub const TRAIL_RATE: f32 = 30.0;

/// Seconds the free look around the ship holds after the mouse stopped, before it springs back.
pub const ORBIT_HOLD: f32 = 1.6;

/// Rate at which the free look around the ship springs back to the nose.
pub const ORBIT_RETURN: f32 = 3.0;

/// Seconds without camera input before the yaw starts to drift behind the direction of travel.
pub const FOLLOW_DELAY: f32 = 1.2;

/// Rate of that drift when the body moves straight ahead of the camera.
pub const FOLLOW_RATE: f32 = 1.2;

/// Horizontal speed below which the yaw does not drift.
pub const FOLLOW_SPEED: f32 = 1.0;

/// Closest and farthest zoom, as multiples of the boom of the rig.
pub const ZOOM_RANGE: (f32, f32) = (0.5, 2.2);

/// Zoom multiplier of one wheel notch toward the body.
pub const ZOOM_STEP: f32 = 0.88;

/// Rate at which the boom reaches the zoom that was asked for.
pub const ZOOM_RATE: f32 = 10.0;

/// Radius of the sphere that keeps the camera body out of geometry. The near plane (0.1 m) fits inside it.
pub const RADIUS: f32 = 0.3;

/// Longest boom that stays inside the capsule of the body: the one from the neck to the eyes.
pub const HEAD_REACH: f32 = 0.3;

/// Radius of the sphere cast along such a boom. The near plane (0.05 m) fits inside it.
pub const HEAD_RADIUS: f32 = 0.12;

/// Gap kept between that sphere and what it touched.
pub const SKIN: f32 = 0.02;

/// Sideways shifts of the casts that measure an obstacle, near and far. An obstacle is narrow
/// when both far spheres pass it and at least one near sphere does: one narrower than 0.3 m
/// always is, one wider than 0.75 m never is.
pub const THIN_SHIFT: (f32, f32) = (0.45, 0.9);

/// How far past the hit the shifted spheres must travel freely.
pub const THIN_PASS: f32 = 1.0;

/// How far past the hit the cast resumes. An obstacle deeper than `THIN_HOP - 2 * RADIUS` is solid.
pub const THIN_HOP: f32 = 0.9;

/// Most narrow obstacles one cast steps over.
pub const THIN_SKIPS: usize = 2;

/// The whiskers: angle around the up axis, and the share of the free length a hit leaves alone.
pub const WHISKERS: [(f32, f32); 4] = [(-0.5, 0.75), (-0.22, 0.4), (0.22, 0.4), (0.5, 0.75)];

/// Boom length below which the whiskers are not cast.
pub const WHISKER_MIN: f32 = 0.6;

/// Rate at which the camera pulls in when something blocks the view: 99 percent in 0.1 s.
pub const PULL_IN: f32 = 45.0;

/// Rate at which the camera eases back out once the view is free.
pub const EASE_OUT: f32 = 2.5;

/// Fastest the camera eases back out, in metres per second.
pub const EASE_SPEED: f32 = 4.0;

/// Reach of the upward cast that looks for a ceiling.
pub const CEILING_REACH: f32 = 4.0;

/// Free height above the body centre under which the rig is fully indoors, and above which it is outdoors.
pub const CEILING_BAND: (f32, f32) = (1.3, 3.2);

/// Rate at which the rig takes its indoor shape.
pub const INDOOR_RATE: f32 = 3.0;

/// Share of the boom the indoor rig gives up.
pub const INDOOR_SHORTEN: f32 = 0.35;

/// Drop of the pivot of the indoor rig.
pub const INDOOR_DROP: f32 = 0.2;

/// Share of the boom lost when looking fully up, and gained when looking fully down.
pub const TILT_REACH: f32 = 0.3;

/// Drop of the camera when looking fully up, and rise when looking fully down.
pub const TILT_DROP: f32 = 0.25;

/// Share of the field of view gained when looking fully up, and lost when looking fully down.
pub const TILT_FOV: f32 = 0.08;

/// Distances from the camera to the head between which the body fades in.
pub const FADE_BAND: (f32, f32) = (0.5, 1.3);

/// Seconds the camera lags behind a moving body, until the lag reaches the limit of the rig.
pub const LAG_TIME: f32 = 0.12;

/// Height of the head bob at walking pace.
pub const BOB: f32 = 0.035;

/// Metres walked per step.
pub const STRIDE: f32 = 2.4;

/// Speed at which the head bob is full.
pub const BOB_SPEED: f32 = 5.0;

/// Rate at which the head bob comes and goes.
pub const BOB_RATE: f32 = 8.0;

/// Fall speed above which a landing dips the camera.
pub const LAND_SPEED: f32 = 3.0;

/// Depth of the landing dip per metre per second of fall, and its limit.
pub const DIP: (f32, f32) = (0.012, 0.18);

/// Largest bank of the body the camera follows, the bank limit of the wings. The rig shows 0.3
/// of it, so the horizon tilts at most about 17 degrees and stays readable.
pub const BANK_LIMIT: f32 = 1.0;

/// Rate at which the camera rolls into a bank.
pub const ROLL_RATE: f32 = 6.0;

/// Largest angle the flight camera turns toward the flight path.
pub const LEAD_LIMIT: f32 = 0.21;

/// Rate at which the flight camera turns toward the flight path.
pub const LEAD_RATE: f32 = 3.0;

/// Airspeed that opens 63 percent of the flight widening.
pub const FLY_SWIFT: f32 = 30.0;

/// Ship speed that opens 63 percent of the cruise widening.
pub const SHIP_SWIFT: f32 = 250.0;

/// Ship speed that opens 63 percent of the warp widening, and how many cruise widenings the warp adds.
pub const WARP: (f32, f32) = (50_000.0, 1.5);

/// Rate at which the widening follows the speed.
pub const RUSH_RATE: f32 = 2.0;

/// Widest field of view the rig asks for.
pub const MAX_FOV: f32 = 2.1;

/// Share of the way the chase camera turns from the nose to the velocity, in atmosphere at speed.
pub const CHASE_ALONG: f32 = 0.5;

/// Speeds between which the chase camera starts to follow the velocity.
pub const CHASE_BAND: (f32, f32) = (20.0, 120.0);

/// Share of the way the chase camera levels from the roof of the ship to the local up near a surface.
pub const CHASE_LEVEL: f32 = 0.85;
