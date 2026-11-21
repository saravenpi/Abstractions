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

/// Centre of the helmet above the neck joint, and its half extents.
pub const HEAD: (f32, [f32; 3]) = (0.19, [0.175, 0.16, 0.15]);

/// Centre of the chest above the waist joint, and its half extents: where the wingsuit takes root.
pub const CHEST: (f32, [f32; 3]) = (0.185, [0.17, 0.215, 0.115]);

/// Half extents of the pelvis core.
pub const HIPS: [f32; 3] = [0.135, 0.075, 0.1];

/// Centre of a boot relative to its ankle, and its half extents: a big block at the end of the leg.
pub const BOOT: ([f32; 3], [f32; 3]) = ([0.0, -0.045, -0.035], [0.085, 0.065, 0.13]);

/// Half width and half height of the lit screen on the head. The picture is twice as wide as tall.
pub const SCREEN: [f32; 2] = [0.116, 0.058];

/// Radius of the booster nozzle in each sole, and where its axis crosses the sole behind the ankle.
pub const NOZZLE: (f32, f32) = (0.045, 0.015);

/// sRGB colour of the armour plates: bone, like the companion.
pub const SHELL: [f32; 3] = [0.93, 0.91, 0.85];

/// sRGB colour of the secondary plates: the companion's slate, dark enough to part from the
/// bone at eight meters.
pub const SHADE: [f32; 3] = [0.47, 0.56, 0.64];

/// sRGB colour of the frame: the limbs, the joints, the visor box and the soles.
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

/// Impact speeds in meters per second between which a soft landing goes from half its depth,
/// a step down, to all of it.
pub const CROUCH_SOFT: (f32, f32) = (2.0, 9.0);

/// How much slower the knees push the body back up after the hardest landing than after a soft
/// one: the recovery takes 0.3 s soft and 0.5 s hard.
pub const RECOVER_SLOW: f32 = 0.6;

/// How much slower the knees give way under the hardest landing than under a soft one, while the
/// hands go down to the ground.
pub const GIVE_SLOW: f32 = 1.0;

/// How deep the knees fold when a jump starts, in meters, and for how many seconds: a short
/// dip while the body already rises, so the jump itself is never delayed.
pub const PUSH_OFF: (f32, f32) = (0.035, 0.09);

/// Fastest the climb speed the air poses read may change, in meters per second squared:
/// well above any fall, so only a bounce off the ground is spread over a few frames.
pub const RISE_RATE: f32 = 60.0;

/// Forward lean per meter per second squared the ground speed gains, in radians, and the most
/// it adds or takes: the body leans into a start and back into a stop.
pub const LEAN_GAIN: (f32, f32) = (0.01, 0.06);

/// Bank into a turn on the ground per radian per second of turn and meter per second of
/// speed, in radians, and the most it banks.
pub const TURN_BANK: (f32, f32) = (0.012, 0.09);

/// Seconds before the ground under a falling body within which the legs reach for it: they
/// start to reach at the longer time and are fully out at the shorter one.
pub const REACH_TIME: (f32, f32) = (0.12, 0.45);

/// Fall speeds in meters per second between which a falling body goes from the upright fall
/// of a jump into the skydiver arch, belly to the ground: a hop never reaches the first, a drop
/// of about 40 m under standard gravity reaches the second.
pub const ARCH_SINK: (f32, f32) = (14.0, 28.0);

/// Seconds before the ground between which a body in the arch turns upright to land: upright
/// at the first, still flat at the second. Long enough that the turn of a quarter turn never
/// jumps a joint, and over before the legs reach out at [`REACH_TIME`].
pub const ARCH_GROUND: (f32, f32) = (0.45, 1.5);

/// Seconds the body takes at least to turn from the upright fall into the arch, or back.
pub const ARCH_SECONDS: f32 = 0.9;

/// Felt loads in g between which the flying body arches: level at the first, fully arched in
/// a hard pull-up at the second.
pub const ARCH_LOAD: (f32, f32) = (1.3, 3.0);

/// Sines of the descent angle between which the gliding body tucks its arms and legs in:
/// from 20 degrees down to 57.
pub const TUCK_DIVE: (f32, f32) = (0.34, 0.84);

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

/// How far the head turns toward the view: 70 degrees to each side, 40 up and down.
pub const NECK_REACH: (f32, f32) = (1.221_73, 0.698_13);

/// How far past the side reach the head eases back to the front, in radians of view yaw.
pub const NECK_FADE: f32 = 0.6;

/// Spring rate of the head turning toward where it looks, per second.
pub const NECK_RATE: f32 = 8.0;

/// Steepest the body pitches along its way through the water: 70 degrees head down or up.
pub const DIP_LIMIT: f32 = 1.221_73;

/// Speeds through the water between which the body goes from upright to lying along its way.
pub const DIP_SPEED: (f32, f32) = (0.3, 2.5);

/// Spring rate of the body pitching along its way through the water, per second.
pub const DIP_RATE: f32 = 4.0;

/// How far a fully crouched body sinks its pelvis, in meters. Deeper pushes the knees out of the
/// front of the player collider.
pub const SQUAT: f32 = 0.22;

/// Forward lean of the whole body and bend of the torso over the pelvis when fully crouched, in
/// radians: 13 degrees in all, a lean for balance. The neck takes the sum back, so the head stays level.
pub const HUNCH: (f32, f32) = (0.15, 0.08);

/// How far behind the hips a fully crouched body sets its feet, in meters: the hips sit back over
/// the heels, which keeps the bent knees inside the player collider.
pub const HEELS: f32 = 0.06;

/// Share of the step height and reach a fully crouched stride keeps: the steps are short.
pub const SHORT_STEP: f32 = 0.6;
