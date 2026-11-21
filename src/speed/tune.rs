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

/// Metres of height per kelvin by which the dew point of rising air falls behind its
/// temperature: the cloud base sits about 125 m higher for each kelvin between the two at the
/// ground (<https://en.wikipedia.org/wiki/Cloud_base>).
pub const DEW_SPREAD: f32 = 125.0;

/// Kelvin between the temperature and the dew point under which the low pressure of a
/// transonic hull or a wing tip still cools the air to its dew point and condenses it, and at
/// which it condenses fully.
pub const COOLING: (f32, f32) = (4.0, 0.5);

/// Slope of the mixing line of the exhaust with the air, in pascals per kelvin, for each pascal
/// of the pressure of the air: `cp EI / (eps Q (1 - eta))`, with the water the burning gives off,
/// 1.25 kilograms per kilogram of a kerosene that holds 43.2 MJ per kilogram, at a propulsive
/// efficiency of 0.3. It is 1.67 Pa/K at 250 hPa
/// (<https://www.satcorps.smce.nasa.gov/site/doc-library/148-Duda_Minnis.JAMC.I.09.pdf>).
pub const MIXING: f64 = 6.67e-5;

/// Kelvin under the Schmidt-Appleman threshold at which a contrail is full: just under it the
/// plume only just condenses.
pub const CONTRAIL_DEPTH: f32 = 3.0;

/// Air density as a share of the home sea level density under which a contrail is too thin to
/// see, and at which it is full: a rocket trail fades out above 40 km on the Earth.
pub const CONTRAIL_AIR: (f32, f32) = (0.002, 0.02);

/// Seconds a contrail lasts in dry air and in air as wet as a covered sky: in dry air the ice
/// sublimates within seconds, in air saturated over ice it spreads and stays for many minutes,
/// of which the ribbon keeps the first.
pub const CONTRAIL_LIFE: (f32, f32) = (4.0, 45.0);
