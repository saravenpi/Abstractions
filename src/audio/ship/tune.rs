//! Tuning constants of the sound of a ship. Frequencies are in hertz, times in seconds,
//! lengths in metres. A level is the root mean square of the layer at its loudest, against a
//! full scale of 1, unless it says otherwise.

/// Density of the air at sea level on the home planet, in kilograms per cubic metre.
pub const SEA_AIR: f32 = 1.2;

/// Helmholtz number `f D / c` at which the mixing noise of a hot supersonic jet peaks.
pub const HELMHOLTZ: f32 = 0.2;

/// Thrust level under which the jet stops narrowing: the flame of an idling engine.
pub const THROTTLE_FLOOR: f32 = 0.15;

/// Exit diameter of a main nozzle and of a lift nozzle, in metres: twice the radii of the model.
pub const DIAMETERS: (f32, f32) = (0.6, 0.25);

/// Thrust level of the main engines with the boost: the level the pumps turn fastest at.
pub const FULL_THRUST: f32 = 1.73;

/// Thrust level of the flame that keeps a running engine lit with the lever at idle.
pub const IDLE_THRUST: f32 = 0.03;

/// Cosine of the angle from the exhaust at which the mixing noise is loudest, 50 degrees, and
/// the width of the lobe around it, in cosine.
pub const LOBE: (f32, f32) = (0.64, 0.45);

/// Share of the mixing noise that reaches a listener ahead of the nose.
pub const LOBE_FLOOR: f32 = 0.18;

/// Resonance of the band of the mixing noise: 0.5 gives the 6 decibels per octave either side
/// of the peak that measured jet spectra show.
pub const MIX_Q: f32 = 0.5;

/// Level of the mixing noise of a jet at full thrust, on its lobe, at the reference distance.
pub const MIX_LEVEL: f32 = 0.22;

/// How many times the peak frequency the noise of the fine turbulence near the nozzle sits at,
/// its highest frequency, its resonance and its level.
pub const FINE: (f32, f32, f32, f32) = (8.0, 3200.0, 0.7, 0.07);

/// Air thickness, as a share of the home sea level, over which the shock cells of the plume
/// and their noise fade in.
pub const SHOCK: (f32, f32) = (0.1, 0.6);

/// Helmholtz number of the noise of the shock cells, its level, and the share of it lost
/// toward the lobe: it radiates to the sides and ahead.
pub const SHOCK_NOISE: (f32, f32, f32) = (1.2, 0.06, 0.6);

/// Resonance of the band of the shock noise and of the plume hitting the ground.
pub const EXTRA_Q: f32 = 2.0;

/// Thrust levels between which the crackle of a jet in air rises from nothing to full.
pub const CRACKLE: (f32, f32) = (0.6, 1.2);

/// Share of the samples that carry a shock at full crackle, the peak of a shock against full
/// scale, and the corner of the decay after its rise.
pub const CRACKLE_SHAPE: (f32, f32, f32) = (0.004, 0.35, 3500.0);

/// Corner and resonance of the rumble the engines send through the airframe, and its level at
/// full thrust.
pub const RUMBLE: (f32, f32, f32) = (85.0, 0.8, 0.1);

/// Rates of the fast and the slow swell of a jet, from the eddies that pass, and the depth of
/// each as a root mean square share of the level.
pub const SURGE: [(f32, f32); 2] = [(11.0, 0.3), (1.3, 0.22)];

/// Share of the lobe a listener gets of the lift nozzles, which point at the ground.
pub const LIFT_LOBE: f32 = 0.6;

/// Height in metres under which the plumes of the lift nozzles reach the ground.
pub const GROUND_REACH: f32 = 14.0;

/// What the ground adds to the lift noise on it: the gain of the reflected low end, and the
/// centre and the level of the harsh band of the plumes that hit it.
pub const IMPINGE: (f32, f32, f32) = (1.0, 900.0, 0.12);

/// Shaft speed of a pump at full speed, in turns per second: 25 000 turns per minute.
pub const SHAFT_HZ: f32 = 420.0;

/// Share of its full speed a pump idles at.
pub const PUMP_IDLE: f32 = 0.4;

/// Seconds a pump takes to cover 63 percent of a rise and of a fall of its speed: the inertia
/// of the rotor.
pub const PUMP_LAG: (f32, f32) = (0.9, 2.2);

/// Level of the second and third shaft orders against the first, and the blade count.
pub const ORDERS: (f32, f32, f32) = (0.35, 0.2, 6.0);

/// Speed of the second pump against the first: two rotors never turn alike, so they beat.
pub const TWIN: f32 = 1.007;

/// Level of one shaft order through the airframe and of the blade whine in the air, at full speed.
pub const PUMP_LEVEL: (f32, f32) = (0.045, 0.02);

/// Rate of the wander of the pump speed, and its depth as a share of the speed.
pub const WANDER: (f32, f32) = (0.6, 0.004);

/// Pump speeds between which the chambers light as the pumps come up.
pub const LIGHT: (f32, f32) = (0.18, 0.3);

/// Share of the thrust of the lift nozzles the pumps feed as they feed the main engines.
pub const LIFT_FEED: f32 = 0.6;

/// Centre of the wind over the canopy at rest, what each metre per second adds, and its top.
pub const WIND_HZ: (f32, f32, f32) = (220.0, 5.0, 2600.0);

/// Resonance of the wind band and its level at the full dynamic pressure.
pub const WIND: (f32, f32) = (0.6, 0.12);

/// Dynamic pressure in pascals at which the wind over the canopy is at its level.
pub const FULL_PRESSURE: f32 = 7.5e4;

/// Corner of the thuds of the buffet and their level in the roughest air.
pub const BUFFET: (f32, f32) = (45.0, 0.22);

/// Centre and resonance of the rattle of the airframe, the share of the samples that carry a
/// knock, and the peak of a knock in the roughest air.
pub const RATTLE: (f32, f32, f32, f32) = (1700.0, 6.0, 0.002, 0.05);

/// Corner in hertz of the deep roar of an entry through the hull, and its level at full heat.
pub const ROAR: (f32, f32) = (70.0, 0.3);

/// Rate of the gusts on the canopy and their depth.
pub const GUST: (f32, f32) = (0.8, 0.35);

/// How rough the passage of Mach 1 is in thick air, and the width of the band it is felt in.
pub const TRANSONIC: (f32, f32) = (0.55, 0.08);

/// Mach numbers between which the cabin outruns the sound of its own engines in the air.
pub const OUTRUN: (f32, f32) = (0.95, 1.15);

/// Lowest tone of the warp drive and what a full spool adds to it.
pub const WARP_HZ: (f32, f32) = (55.0, 275.0);

/// Beat between the two tones of each partial of the drive: at rest, and added by a full spool.
pub const WARP_BEAT: (f32, f32) = (0.7, 6.0);

/// The partials of the drive: frequency against the lowest tone, and level.
pub const WARP_PARTIALS: [(f32, f32); 3] = [(1.0, 1.0), (1.5, 0.5), (2.01, 0.3)];

/// Level of the lowest partial of the drive fully spooled.
pub const WARP_LEVEL: f32 = 0.07;

/// Spool shares between which the bed of the cruise fades in.
pub const CRUISE: (f32, f32) = (0.9, 1.0);

/// Centre of the bed of the cruise, its resonance, how far it wanders and its level.
pub const BED: (f32, f32, f32, f32) = (420.0, 4.0, 0.3, 0.038);

/// Travel in shares per second under which a mechanism counts as stopped.
pub const MOTOR_PACE: f32 = 0.05;

/// Tone of the gear motor and of the canopy servo at speed.
pub const MOTOR_HZ: (f32, f32) = (140.0, 260.0);

/// Share of its tone a motor starts at, the level of its third harmonic, of its hiss, and
/// its level.
pub const MOTOR: (f32, f32, f32, f32) = (0.6, 0.3, 0.5, 0.025);

/// Seconds a motor takes to come up to speed and to stop.
pub const MOTOR_SPIN: f32 = 0.08;

/// Tone and resonance of the thump of a lock, and of the clack of a latch.
pub const KNOCKS: [(f32, f32); 2] = [(70.0, 3.0), (950.0, 5.0)];

/// Peak of the thump of the gear locking down, of the gear locking up, and of a latch.
pub const LOCK: (f32, f32, f32) = (0.22, 0.14, 0.05);

/// Peak of the thump of the canopy shutting, of its latch, and the puff of its seal.
pub const SEAL: (f32, f32, f32) = (0.12, 0.05, 0.6);

/// Peak of the thump of the chambers lighting, and its puff.
pub const IGNITE: (f32, f32) = (0.2, 1.0);

/// Peak of the thump of the lightest and of the hardest landing, the share of it the gear gives
/// back as it settles, the seconds until it does, and the puff of the struts.
pub const LANDING: (f32, f32, f32, f32, f32) = (0.12, 0.45, 0.4, 0.22, 0.8);

/// Centre and resonance of the hiss of the attitude jets and of a seal, the level of the jets
/// at full fire, and the level of a full puff.
pub const HISS: (f32, f32, f32, f32) = (2800.0, 0.9, 0.05, 0.08);

/// Seconds a puff takes to die, and seconds its level takes to follow.
pub const PUFF: (f32, f32) = (0.25, 0.005);

/// Share of the knocks and motors heard in the air outside, and of the jets heard in the hull.
pub const SHARES: (f32, f32) = (0.5, 0.4);

/// Ticks per second of nozzles at full heat with the engines off, the lowest tone of a tick,
/// the span of tones above it, the resonance and the peak.
pub const EMBER: (f32, f32, f32, f32, f32) = (6.0, 1800.0, 2700.0, 9.0, 0.03);

/// Centre and resonance of the breath of the cabin fan, and its level.
pub const FAN: (f32, f32, f32) = (480.0, 0.8, 0.008);

/// Blade tone of the fan and its level, and the tone of the inverter and its level.
pub const BED_TONES: (f32, f32, f32, f32) = (233.0, 0.003, 400.0, 0.0015);

/// Share of the cabin bed that stays with the engines off: the avionics run on the battery.
pub const STANDBY: f32 = 0.6;

/// Corner of the canopy shut and open: glass passes the low end and stops the rest.
pub const CANOPY_HZ: (f32, f32) = (260.0, 6000.0);

/// Gain of the air outside into the cabin with the canopy shut and open. The seat is 4 metres
/// ahead of the nozzles.
pub const LEAK: (f32, f32) = (0.5, 2.5);

/// Highest density of the air, as a share of the home sea level, that makes a jet louder.
pub const AIR_CAP: f32 = 2.0;

/// Distance in metres under which a ship gets no louder, and the reference distance the
/// levels are given at. Past it the pressure falls as `1 / r`, as in free air. The reference
/// is far, because an engine is the loudest thing there is: a pass 100 metres away must rise
/// over a calm day, and a listener 20 metres from a burn gets all the mix can give.
pub const NEAR: (f32, f32) = (13.5, 45.0);

/// Gain of the rumble of the lift nozzles in the seat: they sit in the belly under the cabin.
pub const LIFT_BODY: f32 = 1.9;

/// Distance in metres past which a ship is not heard.
pub const EARSHOT: f32 = 2000.0;

/// Corner of the air over one kilometre at sea level at home: absorption reaches 3 decibels there.
pub const ABSORB_HZ: f32 = 1370.0;

/// Lowest and highest corner the air between a ship and the listener makes.
pub const DULLEST_HZ: f32 = 180.0;

/// Corner of air that absorbs nothing audible.
pub const OPEN_HZ: f32 = 18_000.0;

/// Seconds of sound a ship keeps on its way to the listener: earshot at 330 metres per second.
pub const TRAVEL: f32 = 6.2;

/// Seconds the delay of a ship takes to follow its distance, through two stages.
pub const DELAY_GLIDE: f32 = 0.06;

/// Most samples the delay may shrink or grow by per sample: sound is never heard backwards.
pub const MAX_PACE: f32 = 0.9;

/// Height in metres of the ears and of the nozzles, and the share the ground reflects.
pub const BOUNCE: (f32, f32, f32) = (1.7, 1.25, 0.45);

/// Highest level the ships reach together: the room left under it is for voices and cues.
pub const BUS_CEILING: f32 = 0.5;

/// Level of the whoosh in the middle of a cloud.
pub const WHOOSH_LEVEL: f32 = 0.19;
