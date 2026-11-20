//! Tuning of the speed cues: pressures in pascals, speeds in metres per second, shares from 0 to 1.

/// Dynamic pressures in pascals between which a body flying in the air starts to buffet and buffets
/// fully: 300 Pa is a wingsuit at 22 m/s in home air at sea level, 6000 Pa is one at 100 m/s.
pub const BUFFET: (f32, f32) = (300.0, 6.0e3);

/// Share of the shake of a ship in rough air that a body in the wingsuit gets at full buffeting. The
/// view of the pilot moves 12 mm at most, the view of the glider half of that.
pub const BODY_SHARE: f32 = 0.5;

/// Rate per second at which the buffeting level follows the air.
pub const BUFFET_RATE: f32 = 3.0;

/// How wet the air is, from 0 (dry) to 1 (inside a cloud or in rain), between which the low
/// pressure of a wingtip vortex starts to condense and condenses fully.
pub const MOIST: (f32, f32) = (0.15, 0.6);

/// Brightness a fully condensing vortex adds to a wingtip trail, as a share of the trail it has in dry air.
pub const CONDENSE_GAIN: f32 = 0.8;

/// Mach numbers between which the vapour cone of a ship in humid air rises from nothing to full. It
/// is full at Mach 1.
pub const CONE_BAND: (f32, f32) = (0.85, 1.0);

/// Mach number above which the vapour cone is gone again: the ship has left the transonic band.
pub const CONE_PEAK: f32 = 1.25;

/// Air density as a share of the home sea level density under which no cone forms.
pub const CONE_AIR: (f32, f32) = (0.05, 0.4);

/// Highest alpha of the vapour cone.
pub const CONE_ALPHA: f32 = 0.35;
