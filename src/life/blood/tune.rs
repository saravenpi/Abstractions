//! Tuning of blood: the numbers of the body and the fluid. Units are SI unless a line says otherwise.

/// Share of the body mass that is blood, unitless: mammals carry 6 to 8 %.
pub const SHARE: f32 = 0.07;

/// Density of blood in kilograms per cubic metre.
pub const DENSITY: f32 = 1060.0;

/// Pressure that drives blood through the capillaries of a body at ground level, in pascals: 60 mmHg.
pub const BASE: f32 = 8000.0;

/// Share of the hip height that the head stands above the heart, unitless.
pub const LIFT: f32 = 0.6;

/// Share of the closed-circuit pressure an open circulation (blood that bathes the organs) keeps,
/// unitless.
pub const OPEN: f32 = 0.1;

/// Pressure in a vein or a capillary bed that a shallow cut opens, in pascals: 15 mmHg.
pub const VENOUS: f32 = 2000.0;

/// Discharge coefficient of a sharp-edged opening, unitless.
pub const DISCHARGE: f32 = 0.6;

/// Diameter of the aorta of a body of one kilogram, in metres; it grows with the mass to the power
/// [`VESSEL_EXPONENT`]. Our estimate, fitted to 2.5 cm at 70 kg.
pub const VESSEL: f32 = 0.0048;

/// Exponent of the mass in the diameter of the aorta, unitless.
pub const VESSEL_EXPONENT: f32 = 0.36;

/// Share of the face of a shallow cut that is the mouth of vessels, unitless.
pub const PORES: f32 = 0.005;

/// Share of the blood that, once lost, kills, unitless: class IV haemorrhage starts near 40 %.
pub const LETHAL: f32 = 0.4;

/// Heart rate of a body of one kilogram, in beats per second: 241 beats per minute, falling with the
/// mass to the power -1/4 (allometric scaling of the heart rate of mammals).
pub const HEART: f32 = 4.0;

/// Seconds in which a shallow wound clots to 1/e of its flow.
pub const CLOT: f32 = 90.0;

/// Seconds in which a wound that reached a major vessel clots to 1/e of its flow: a vessel that is
/// open does not seal by itself, so the flow falls with the pressure alone for a long while.
pub const CLOT_MAJOR: f32 = 900.0;

/// Thickness of the film a drop spreads into on the ground, in metres.
pub const FILM: f32 = 0.001;

/// Share of the oxygenated colour that blood keeps once it dried on the ground, unitless.
pub const DRIED: f32 = 0.3;

/// Density of the air at home, in kilograms per cubic metre.
pub const AIR: f32 = 1.2;

/// Drag coefficient of a sphere in air, unitless.
pub const SPHERE: f32 = 0.47;
