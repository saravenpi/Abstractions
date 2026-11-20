//! Tuning of the senses: how fast an animal notices a robot, how far it sees, hears and smells, and
//! when it stands its ground.
//!
//! Detection builds up over time rather than firing at once, faster the nearer the threat, as in
//! stealth games built on a meter (Splinter Cell Blacklist, GDC 2014). Scent travels downwind in a
//! narrow plume; hunters smell farther than grazers.

/// Metres under which the meter fills in [`FILL_NEAR`] seconds.
pub const NEAR: f32 = 10.0;

/// Metres under which the meter fills in [`FILL_MID`] seconds.
pub const MID: f32 = 30.0;

/// Metres under which the meter fills in [`FILL_FAR`] seconds, and the farthest an animal sees.
pub const FAR: f32 = 60.0;

/// Seconds the meter takes to fill under [`NEAR`] metres.
pub const FILL_NEAR: f32 = 0.3;

/// Seconds the meter takes to fill from [`NEAR`] to [`MID`] metres.
pub const FILL_MID: f32 = 1.5;

/// Seconds the meter takes to fill past [`MID`] metres, by sight to [`FAR`], by ear or nose beyond.
pub const FILL_FAR: f32 = 4.0;

/// Fall of the meter per second.
pub const DRAIN: f32 = 0.1;

/// Speed in metres per second under which a robot counts as still or creeping: the meter fills at
/// half the rate.
pub const STILL: f32 = 0.4;

/// Speed in metres per second over which a threat counts as fast: the meter fills at twice the rate.
pub const FAST: f32 = 4.0;

/// Loudness over which a threat counts as loud as a fast one: boots or a landing.
pub const LOUD: f32 = 2.0;

/// Awareness, 0 to 1, from which an animal is wary.
pub const WARY: f32 = 0.3;

/// Awareness, 0 to 1, from which an animal flees or stands its ground.
pub const ALARM: f32 = 0.8;

/// Steps of the ray of sight over the height field.
pub const RAY_STEPS: u32 = 5;

/// Metres above its root at which a robot is seen: about its chest.
pub const ROBOT_CHEST: f32 = 1.0;

/// Walking speed in metres per second of a robot, the speed of loudness 1.
pub const WALK: f32 = 1.5;

/// Metres a threat of loudness 1 is heard from.
pub const EAR: f32 = 20.0;

/// Loudness of boots at full thrust.
pub const BOOTS: f32 = 4.0;

/// Loudness per metre per second of speed lost into the ground at a landing.
pub const LANDING: f32 = 0.5;

/// Half width in degrees of the downwind cone of scent.
pub const SCENT_HALF: f32 = 20.0;

/// Metres a grazer smells a robot from, downwind.
pub const NOSE_GRAZER: f32 = 80.0;

/// Metres a hunter smells a robot from, downwind.
pub const NOSE_HUNTER: f32 = 120.0;

/// Wind speed in metres per second under which nothing carries scent.
pub const BECALMED: f32 = 0.2;

/// Range factor of the senses of a herd's lookout.
pub const LOOKOUT: f32 = 1.5;

/// Mass in kilograms of a robot, as animals weigh it up.
pub const ROBOT_KG: f32 = 80.0;

/// Ratio of own mass times health over threat mass above which an animal fights.
pub const FIGHT: f32 = 0.5;

/// Share of its own mass above which a threat makes a plant eater flee.
pub const PREY_SHARE: f32 = 0.5;

/// Seconds after a landing on a planet during which no hunter goes for a robot.
pub const GRACE: f32 = 60.0;

/// Metres a robot moves in one frame that count as a landing: a respawn or a ship's arrival.
pub const JUMP: f64 = 50.0;

/// Metres ahead of a fleeing animal its way out is probed.
pub const CORNER_PROBE: f32 = 3.0;

/// Rise of the ground in metres over [`CORNER_PROBE`], per metre of the animal's height, that
/// corners it.
pub const CORNER_RISE: f32 = 1.5;

/// Health in joules of damage per kilogram to the power 0.75: tougher with size, as metabolic rate
/// and tissue scale (Kleiber 1932, "Body size and metabolism", Hilgardia 6, 315-353).
pub const HEALTH: f32 = 30.0;

/// Share of health under which an animal is wounded.
pub const WOUNDED: f32 = 0.4;

/// Share of its speed a wounded animal limps at.
pub const LIMP: f32 = 0.6;

/// Metres between two drops of blood.
pub const BLOOD_STEP: f64 = 2.0;

/// Most drops of blood kept at once; the oldest go first.
pub const BLOOD_MOST: usize = 512;

/// Length in metres of a drop of blood per cube root of a kilogram of the animal.
pub const BLOOD_PER_KG: f32 = 0.04;
