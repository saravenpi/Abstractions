//! Tuning constants of how damage shows on the body. Lengths are metres in the frame of the
//! segment they sit on, times are seconds.

/// Most dents and scorch marks a part carries when it is destroyed.
pub const SCARS: usize = 6;

/// Health under which a part hangs a loose panel.
pub const LOOSE: f32 = 0.55;

/// Health under which a part throws sparks now and then.
pub const SPARKING: f32 = 0.4;

/// Seconds between two chances of a spark from a hurt part.
pub const DRIZZLE: f32 = 0.25;

/// Share of those chances a part at zero health takes, unitless.
pub const DRIZZLE_ODDS: f32 = 0.7;

/// Seconds a spark lives.
pub const SPARK_LIFE: f32 = 0.45;

/// Speed of a spark leaving a blow, in metres per second.
pub const SPARK_SPEED: f32 = 3.0;

/// Share of its speed a spark loses each second, unitless: 4 means it slows to 2 % in a second.
pub const SPARK_DRAG: f32 = 4.0;

/// Size of a spark, in metres.
pub const SPARK_SIZE: f32 = 0.014;

/// Joules per extra spark in a blow, with three sparks at least and fourteen at most.
pub const SPARK_JOULES: f32 = 30.0;

/// Edge of a dent or scorch mark at its smallest and its largest, in metres.
pub const MARK: [f32; 2] = [0.04, 0.09];

/// Thickness of a mark, in metres.
pub const MARK_THIN: f32 = 0.004;

/// Size of a loose panel: width, height and thickness, in metres.
pub const PANEL: [f32; 3] = [0.07, 0.1, 0.008];

/// Swing of a loose panel about its hinge, in radians: the least it hangs open and the most it
/// swings either side of that.
pub const PANEL_SWING: [f32; 2] = [0.5, 0.3];

/// Speed of the swing of a loose panel, in radians per second.
pub const PANEL_RATE: f32 = 3.0;

/// Share of the stride a hurt leg loses at zero health, unitless.
pub const STRIDE_LOSS: f32 = 0.5;

/// Roll of the body toward the sound leg at zero health of the other, in radians.
pub const LIMP_ROLL: f32 = 0.12;

/// Drop of the pelvis at zero health of a leg, in metres.
pub const LIMP_DIP: f32 = 0.03;

/// Forward lean of a crawling body, in radians.
pub const CRAWL_LEAN: f32 = 0.6;

/// Ground speed at which a limp is at its fullest, in metres per second.
pub const LIMP_SPEED: f32 = 3.0;

/// Shoulder swing, lift and elbow bend of a broken arm hanging, in radians.
pub const HANG: [f32; 3] = [0.08, 0.12, 0.15];

/// Flickers of a hurt screen per second.
pub const FLICKER: f32 = 9.0;

/// Share of the flickers a head at zero health drops out in, unitless.
pub const DROPOUT: f32 = 0.6;

/// Share of its light the screen loses at zero health of the head, unitless.
pub const DIM: f32 = 0.6;

/// Share of its light a screen keeps in a dropout, unitless.
pub const LOST: f32 = 0.05;

/// Least impulse that staggers the robot, in metres per second: under it a blow only hurts.
pub const STAGGER_MIN: f32 = 0.4;

/// Impulse above which the robot loses its footing and falls, in metres per second, for a robot on
/// its feet, not crouched and not braced.
pub const FALL_AT: f32 = 3.0;

/// Impulse a robot on its feet absorbs without losing its balance for long, in metres per second:
/// it sets the recovery time. A crouch doubles it and a block on the side of the blow adds [`BRACE`].
pub const CAPACITY: f32 = 2.0;

/// Share of the capacity a block on the side of the blow adds, unitless.
pub const BRACE: f32 = 0.5;

/// Mass of the arm behind a guard, in kilograms: the share of a blow the tool takes is its mass over
/// its mass and this, as in the thrust speed of the tool.
pub const ARM_KG: f32 = 1.0;

/// Most of a blow a guard takes, unitless.
pub const ABSORB_MAX: f32 = 0.8;

/// Seconds a robot takes to recover from an impulse of the size of [`CAPACITY`].
pub const RECOVER_S: f32 = 0.35;

/// Least and most the recovery stretches by, in shares of [`RECOVER_S`].
pub const RECOVER_SPAN: [f32; 2] = [0.5, 3.0];

/// Lean of the body at the deepest of a stagger that stops short of a fall, in radians.
pub const LEAN_MAX: f32 = 0.35;

/// Seconds the lean takes to build.
pub const RISE: f32 = 0.08;

/// Seconds a fall takes to reach the ground, and the seconds the robot then takes to get up.
pub const DOWN: [f32; 2] = [0.25, 0.6];

/// Seconds the robot lies on the ground between falling and getting up.
pub const LIE: f32 = 0.4;

/// Pitch of a body that lies on the ground, in radians.
pub const LYING: f32 = 1.4;

/// Drop of the pelvis at the deepest of a stagger, in metres.
pub const KNEES: f32 = 0.06;

/// Share of the direction of a blow along the side of the robot from which a guard on that side
/// counts as meeting it, unitless.
pub const GUARD_COS: f32 = 0.3;

/// Fastest impulse the robot is shoved at, in metres per second.
pub const SHOVE_MAX: f32 = 8.0;
