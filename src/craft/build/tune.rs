//! Tuning constants of builds: the robot's arm and the shapes no measurement pins down yet.

use std::f32::consts::PI;

/// Swing constant C of the tip speed law v = C / (I + I_hand)^0.27, in m/s·(kg·m²)^0.27: a 0.32
/// kg·m² build reaches 14.4 m/s at 0.6 m, about 24 rad/s.
pub const TIP_C: f32 = 10.8;

/// Exponent of the tip speed law, unitless (Cross and Bower).
pub const TIP_POWER: f32 = 0.27;

/// Moment of inertia of the hand and wrist about the grip, in kg·m².
pub const HAND_INERTIA: f32 = 0.02;

/// Torque one hand holds level, in newton metres.
pub const ONE_HAND: f32 = 15.0;

/// Torque two hands hold level, in newton metres.
pub const TWO_HANDS: f32 = 30.0;

/// Effective mass of the arm behind a thrust or a throw, in kilograms.
pub const ARM_MASS: f32 = 1.0;

/// Work the arm puts into a thrust, in joules.
pub const THRUST_ENERGY: f32 = 60.0;

/// Work the arm puts into a throw, in joules.
pub const THROW_ENERGY: f32 = 400.0;

/// Arc a swing accelerates over, in radians.
pub const SWING_ARC: f32 = PI / 2.0;

/// Wedge angle of an edge, in radians (30°).
pub const EDGE_ANGLE: f32 = PI / 6.0;

/// Wedge angle of a point, in radians (25°).
pub const POINT_ANGLE: f32 = 25.0 * PI / 180.0;

/// Length of edge that bites into a target, in metres.
pub const ENGAGE: f32 = 0.05;

/// Stiffness under which a layer bruises rather than breaks under a blunt blow, in gigapascals.
pub const SOFT: f32 = 1.0;

/// Share of momentum a blow keeps through each layer, unitless (Dwarf Fortress loses about 5 %).
pub const KEPT: f32 = 0.95;

/// Duration of an impact, in seconds: the force on the joints is momentum over this time.
pub const CONTACT: f32 = 0.002;

/// Depth of the socket that holds a head or a counterweight on the shaft, in metres.
pub const SOCKET: f32 = 0.04;

/// Shear strength of a bare friction fit with no binding, in megapascals.
pub const FIT_SHEAR: f32 = 1.0;

/// Share of its strength a binding holds in shear, unitless, kept within 5 to 20 MPa.
pub const BIND_SHARE: f32 = 0.25;

/// Weakest and strongest shear of a sinew or resin binding, in megapascals.
pub const BIND_SHEAR: [f32; 2] = [5.0, 20.0];

/// Exponent of joint life against its safety margin, unitless: life in hits = margin^power.
pub const WEAR_POWER: f32 = 6.0;

/// Least Mohs hardness a material needs to hold an edge or a point.
pub const EDGE_MIN: f32 = 3.0;

/// Drag coefficient of a spear in flight, unitless.
pub const DRAG: f32 = 0.8;

/// Distance a thrown spear is judged at, in metres.
pub const THROW_RANGE: f32 = 15.0;

/// Air density at home sea level, in kilograms per cubic metre.
pub const SEA_AIR: f64 = 1.225;

/// Head masses the solver tries per intent (cut, pierce, crush, throw), in kilograms.
pub const HEAD_MASSES: [[f32; 3]; 4] = [
    [0.3, 0.6, 1.0],
    [0.1, 0.2, 0.4],
    [0.6, 1.2, 2.0],
    [0.1, 0.2, 0.3],
];

/// Shaft lengths the solver tries per intent, in metres.
pub const SHAFT_LENGTHS: [[f32; 3]; 4] = [
    [0.4, 0.6, 0.8],
    [0.8, 1.1, 1.4],
    [0.4, 0.6, 0.9],
    [1.0, 1.2, 1.4],
];

/// Grip length the solver uses, in metres.
pub const GRIP: f32 = 0.12;

/// Shaft diameter per metre of shaft the solver uses, unitless, kept within [`BORE`].
pub const SLENDER: f32 = 1.0 / 30.0;

/// Thinnest and thickest shaft, in metres.
pub const BORE: [f32; 2] = [0.012, 0.04];

/// Most combinations the solver scores for one request.
pub const COMBINATIONS: usize = 200;
