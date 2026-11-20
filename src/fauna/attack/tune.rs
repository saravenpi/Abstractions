//! Tuning of the attacks and of the dead: wind-ups, reaches, shares of the body that carry a blow,
//! and the fall of a body that died.

/// Seconds of wind-up of an attack by a body of no length.
pub const WIND_BASE: f32 = 0.25;

/// Seconds of wind-up for each square root of a body length over the gravity: the Froude scaling of
/// the gaits, so a 1 m animal on a 22 m/s² world winds up for 0.68 s.
pub const WIND_FROUDE: f32 = 2.0;

/// Recovery after the blow, in wind-ups.
pub const RECOVER: f32 = 1.5;

/// Seconds the blow of a bite, swipe, lunge or sweep takes to arrive.
pub const STRIKE: f32 = 0.18;

/// Seconds a charge or a trample runs.
pub const DASH: f32 = 0.6;

/// Mass in kilograms over which an animal tramples.
pub const TRAMPLE_KG: f32 = 500.0;

/// Neck length over body length over which an animal lunges.
pub const LUNGE_NECK: f32 = 0.6;

/// Body lengths of room a charge needs before the target.
pub const CHARGE_ROOM: f32 = 2.0;

/// Metres past the body length from which an animal charges.
pub const CHARGE_RANGE: f32 = 8.0;

/// Half the angle in radians in front of the animal in which it charges.
pub const CHARGE_CONE: f32 = 0.45;

/// Share of the reach beyond which a blow whiffs.
pub const SLACK: f32 = 1.15;

/// Metres past the body a bite reaches.
pub const MOUTH: f32 = 0.5;

/// Share of the mass that moves with a bite, a lunge, a swipe, a sweep, a charge and a trample.
pub const SHARE: [f32; 6] = [0.5, 0.08, 0.06, 0.1, 0.03, 1.0];

/// Stiffness in gigapascals and mass in kilograms of the plate of the robot that a blow rings: an
/// aluminium alloy, our estimate.
pub const PLATE: (f32, f32) = (70.0, 8.0);

/// Metres from its middle the robot is struck at above its feet: its blows land at the height of
/// the part that struck, within this much of the middle of the body.
pub const SPREAD: f32 = 0.9;

/// Seconds a hit part takes to spring back.
pub const FLINCH: f32 = 0.4;

/// Longest push in body lengths of a hit part.
pub const FLINCH_PUSH: f32 = 0.25;

/// Share of the pace left when every leg is broken.
pub const LAME: f32 = 0.3;

/// Radians the collapse starts leaning at, and its first speed in radians per second.
pub const LEAN: (f32, f32) = (0.12, 0.3);

/// Share of its speed a falling body keeps when it hits the ground.
pub const BOUNCE: f32 = 0.25;

/// Energy in joules, stiffness in gigapascals and mass in kilograms of the flint knife that cuts a
/// part from a carcass, for the sound of the cut.
pub const CARVE: (f32, f32, f32) = (6.0, 30.0, 0.3);

/// Metres from the eyes within which a part of a carcass is cut or a spear pulled.
pub const CUT_RANGE: f32 = 3.5;

/// Metres past the size of a part within which the line of sight cuts it.
pub const CUT_SLACK: f32 = 0.25;

/// How far the hips drop in the wind-up of an attack of full commitment, as a share of the hip height.
pub const CROUCH: f32 = 0.25;

/// Metres of a body length that the weight moves back in the wind-up of an attack of full commitment,
/// and forward again as it is thrown.
pub const WEIGHT: f32 = 0.18;

/// Radians a rearing body pitches nose up in the wind-up of a trample, and of a downward swipe.
pub const REAR: [f32; 2] = [0.6, 0.3];

/// Radians the head end drops in the wind-up of a charge and of a lunge, nose down.
pub const DUCK: [f32; 2] = [0.15, 0.1];

/// Radians a body turns its flank to the target in the wind-up of a tail sweep, and swings its hips
/// back through in the blow.
pub const FLANK: f32 = 0.5;

/// Radians a body is pitched forward, off balance, at the start of its recovery for a full
/// commitment.
pub const OFF_BALANCE: f32 = 0.2;

/// Commitment of each attack, in the order of [`super::kinds::Kind`]: how much of its body goes into
/// it, from 0 to 1. A charge throws the whole weight, a bite only the head.
pub const COMMIT: [f32; 6] = [1.0, 0.3, 0.6, 0.5, 0.4, 0.8];
