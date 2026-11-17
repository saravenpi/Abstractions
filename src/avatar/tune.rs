//! Every number of the robot body the owner may want to tune, in one place.
//!
//! Lengths are meters in the frame of the player body: origin at the capsule centre, +Y up, -Z ahead.

/// Height of the soles at rest: the bottom of the player capsule.
pub const SOLE_Y: f32 = -0.9;

/// Height of the ankle joint above the sole.
pub const ANKLE_RISE: f32 = 0.11;

/// Length of the shin, knee to ankle.
pub const SHIN: f32 = 0.36;

/// Length of the thigh, hip to knee.
pub const THIGH: f32 = 0.36;

/// Where the pelvis segment sits at rest.
pub const PELVIS: [f32; 3] = [0.0, -0.02, 0.0];

/// Where the right hip joint sits in the pelvis. The left one mirrors it.
pub const HIP: [f32; 3] = [0.095, -0.05, 0.0];

/// Where the waist joint of the torso sits in the pelvis.
pub const WAIST: [f32; 3] = [0.0, 0.07, 0.0];

/// Where the right shoulder joint sits in the torso. The left one mirrors it.
pub const SHOULDER: [f32; 3] = [0.212, 0.33, 0.0];

/// Where the neck joint of the head sits in the torso.
pub const NECK: [f32; 3] = [0.0, 0.43, 0.0];

/// Length of the upper arm, shoulder to elbow.
pub const UPPER_ARM: f32 = 0.23;

/// Length of the forearm, elbow to wrist.
pub const FOREARM: f32 = 0.21;

/// Centre of the rounded head above the neck joint, and its half extents.
pub const HEAD: (f32, [f32; 3]) = (0.19, [0.155, 0.145, 0.15]);

/// Centre of the torso above the waist joint, and its half extents.
pub const CHEST: (f32, [f32; 3]) = (0.185, [0.17, 0.215, 0.115]);

/// Half extents of the pelvis block.
pub const HIPS: [f32; 3] = [0.135, 0.075, 0.1];

/// Centre of a boot relative to its ankle, and its half extents: slightly oversized.
pub const BOOT: ([f32; 3], [f32; 3]) = ([0.0, -0.045, -0.035], [0.075, 0.065, 0.13]);

/// Radii of the limbs: upper arm, forearm, thigh and shin.
pub const LIMB_RADIUS: [f32; 4] = [0.04, 0.036, 0.055, 0.046];

/// Radius of the joint caps: shoulder, elbow, wrist, hip, knee and ankle.
pub const CAP_RADIUS: [f32; 6] = [0.052, 0.046, 0.04, 0.066, 0.06, 0.052];

/// Half width and half height of the lit screen on the head. The picture is twice as wide as tall.
pub const SCREEN: [f32; 2] = [0.116, 0.058];

/// Radius of the booster nozzle in each sole, and where its axis crosses the sole behind the ankle.
pub const NOZZLE: (f32, f32) = (0.045, 0.015);

/// sRGB colour of the shell: soft off-white, like the companion.
pub const SHELL: [f32; 3] = [0.93, 0.93, 0.9];

/// sRGB colour of the shaded shell parts: the companion's blue-grey.
pub const SHADE: [f32; 3] = [0.7, 0.78, 0.86];

/// sRGB colour of the joints, the bezel and the nozzles.
pub const DARK: [f32; 3] = [0.09, 0.1, 0.13];

/// sRGB accent colour: the orange of the ship trim.
pub const ACCENT: [f32; 3] = [0.95, 0.48, 0.16];

/// Seconds a change of mode takes to blend in: between poses on the feet or in the air, and when
/// the change turns the whole body over (into or out of the water, the wingsuit or the seat).
pub const BLEND_SECONDS: (f32, f32) = (0.22, 0.6);

/// Stride length at rest and its gain per meter per second, in meters. A stride is two steps.
pub const STRIDE: (f32, f32) = (0.8, 0.24);

/// Shortest and longest stride in meters.
pub const STRIDE_SPAN: (f32, f32) = (0.8, 2.9);

/// Distance in meters a planted foot travels under the body. The collider radius caps it.
pub const REACH: f32 = 0.34;

/// How high a swinging foot lifts at a walk and at a sprint, in meters.
pub const STEP_LIFT: (f32, f32) = (0.06, 0.11);

/// How far the hips sit under their straight-leg height while standing and while striding.
pub const SINK: (f32, f32) = (0.012, 0.05);

/// Height of the stride bob at rest and its gain per meter per second, in meters.
pub const BOB: (f32, f32) = (0.008, 0.003);

/// Forward lean while striding at rest and its gain per meter per second, in radians.
pub const LEAN: (f32, f32) = (0.03, 0.012);

/// Speeds between which the stride turns from a walk into a run, in meters per second.
pub const RUN_SPAN: (f32, f32) = (4.0, 8.5);

/// How deep a soft and a hard landing press the body down, in meters.
pub const CROUCH: (f32, f32) = (0.06, 0.22);

/// Spring rate of the landing crouch in radians per second.
pub const CROUCH_RATE: f32 = 16.0;

/// Forward lean of the prone flight poses in radians, before the flight path pitch.
pub const PRONE: f32 = 1.45;

/// Seconds the wingsuit takes to unfold or fold.
pub const UNFOLD_SECONDS: f32 = 0.3;

/// Flutter of the stretched fabric in meters, at rest and at [`FLUTTER_SPEED`].
pub const FLUTTER: (f32, f32) = (0.004, 0.022);

/// Airspeed in meters per second at which the flutter is strongest.
pub const FLUTTER_SPEED: f32 = 45.0;

/// Length of the boot plume in meters at no thrust and at full thrust.
pub const PLUME: (f32, f32) = (0.05, 0.55);

/// Linear colour of the plume and nozzle glow at full thrust, above 1 so bloom makes it glow.
pub const PLUME_HDR: [f32; 3] = [2.2, 5.5, 9.0];

/// Brightness of the lit screen pixels, as a multiple of white.
pub const SCREEN_HDR: f32 = 2.6;

/// Luminous power of the screen light in lumens at full glow: a third of the companion's.
pub const LIGHT_POWER: f32 = 850.0;

/// Reach of the screen light in meters.
pub const LIGHT_RANGE: f32 = 3.5;

/// Seconds of standing still before the face turns sleepy.
pub const SLEEPY_AFTER: f32 = 45.0;

/// Fall speed in meters per second beyond which the face is wide-eyed.
pub const FREE_FALL: f32 = 9.0;

/// Airspeed in meters per second beyond which the face is focused.
pub const FAST_FLIGHT: f32 = 25.0;

/// Seconds the happy or wincing face stays after a landing.
pub const LANDING_FACE: f32 = 1.3;

/// Landing hardness above which the face winces instead of smiling.
pub const WINCE_ABOVE: f32 = 0.5;
