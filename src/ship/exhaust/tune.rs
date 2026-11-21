//! Tuning constants of the exhaust of a ship. Lengths are in metres, times in seconds.

/// Seconds a main engine takes to spool up to the thrust asked: the chamber fills and the pumps
/// come up to speed. A visual lag: the flight model applies the thrust at once.
pub const SPOOL_UP: f32 = 0.35;

/// Seconds a main engine takes to shut down, from full thrust to nothing.
pub const SPOOL_DOWN: f32 = 0.9;

/// Seconds a nozzle takes to heat to the glow of the thrust it carries.
pub const HEAT_UP: f32 = 0.8;

/// Seconds a nozzle takes to cool from full glow to dark, with the engine off.
pub const HEAT_DOWN: f32 = 5.0;

/// Seconds the reaction-control jets take to open and to close.
pub const RCS_SPOOL: f32 = 0.06;

/// Speed in metres per second of the gas leaving a main nozzle: a specific impulse of 310
/// seconds. The noise of the plume is scrolled along it.
pub const EXHAUST_SPEED: f32 = 3040.0;

/// Most plume lengths per second the noise scrolls: the exhaust speed over the length of the
/// plume would flicker faster than a frame shows, so it is capped where the eye still sees flow.
pub const SCROLL: f32 = 5.0;

/// Length of the bright jet of a nozzle at full thrust, in exit diameters, in thick air and in
/// vacuum, where the plume runs on much farther.
pub const LENGTH: (f32, f32) = (3.0, 8.0);

/// Widest radius of the plume of a nozzle at full thrust, in exit radii, in thick air and in
/// vacuum, where it expands into the lack of pressure around it.
pub const WIDTH: (f32, f32) = (1.0, 2.8);

/// Share of its brightness a plume keeps in vacuum: the same gas spread over a far larger
/// volume shines much less.
pub const FAINT: f32 = 0.3;

/// Length in exit diameters of a warp plume with the drive fully up.
pub const WARP_LENGTH: f32 = 14.0;

/// Length in metres of a reaction-control jet at full thrust.
pub const RCS_LENGTH: f32 = 0.45;

/// Radius in metres of the exit of a reaction-control port.
pub const RCS_EXIT: f32 = 0.035;

/// Air thickness, as a share of the home sea level, over which the shock diamonds fade in.
pub const SHOCK: (f32, f32) = (0.1, 0.6);

/// Thrust level under which a jet is not drawn.
pub const OFF: f32 = 0.02;

/// Light of a main engine at full thrust, in lumens. Its range is what
/// [`reach`](crate::render::reach) gives it.
pub const MAIN_LIGHT: f32 = 90_000.0;
/// Light of the four lift nozzles together at full thrust, in lumens.
pub const LIFT_LIGHT: f32 = 60_000.0;
/// Light of the headlight in the nose, in lumens, the half angle of its beam in radians and the
/// radius of its lens in metres. An aircraft landing light gives some tens of thousands of
/// lumens: this one lights the ground 24 m ahead by the inverse square law without burning it
/// white, and its lens is wide enough for a near surface to take a soft highlight.
pub const HEADLIGHT: (f32, f32, f32) = (40_000.0, 0.45, 0.08);
/// Light of a navigation lamp at the tip of a winglet, in lumens: it lights the winglet and the
/// wing tip around it, 2.9 m by the inverse square law.
pub const TIP_LIGHT: f32 = 600.0;
/// Dynamic pressure in pascals, 25 metres per second in air at sea level, above which the lift
/// thrusters are taken to be shut, the ship flying on its wings and its main engines.
pub const HOVER_PRESSURE: f64 = 400.0;

/// Height in metres under which the lift nozzles raise dust, and the dust of a nozzle at full
/// thrust on the ground, in puffs per second.
pub const DUST: (f32, f32) = (14.0, 9.0);
