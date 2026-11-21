//! Tuning of the drawn animals: how far and how many are drawn, how often the herds are read, and
//! how they move when alarmed. How they notice robots is tuned in `super::senses::tune`.

/// Metres along the ground from the local robot within which animals are drawn.
pub const REACH: f64 = 150.0;

/// Metres past [`REACH`] a drawn animal stays before it fades out, so animals at the edge do not
/// flicker in and out.
pub const SLACK: f64 = 15.0;

/// Most animals drawn at once.
pub const MOST: usize = 48;

/// Metres from the local robot within which an animal gets full detail: the ground under every
/// foot, leg solvers and chains.
pub const NEAR: f32 = 40.0;

/// Metres from the eyes of the local robot within which it sees a herd it looks at: the reach animals
/// are drawn within, so a herd is never marked from animals the player could not see.
pub const SIGHT: f64 = REACH;

/// Half the angle of view in radians within which the robot sees a herd, about the field of view of
/// the camera.
pub const VIEW: f64 = 0.9;

/// Seconds a herd stays marked at the place it was last seen, fading out.
pub const MEMORY: f64 = 180.0;

/// Most herds marked at once.
pub const MARKS: usize = 3;

/// Shortest body in metres of a species whose herds are marked: smaller animals are hard to see
/// from the mark.
pub const MARK_SIZE: f32 = 0.4;

/// Metres under which a herd is left unmarked beside a nearer marked one.
pub const MARK_APART: f64 = 60.0;

/// Metres along the ground of the search radius of a watch: twice [`SIGHT`], past [`REACH`] and
/// [`SLACK`], plus the room the robot walks before the watch is rolled again.
pub const WATCH_RADIUS: f64 = 2.0 * SIGHT + WATCH_MOVE;

/// Metres the robot walks from the search point of a watch before it is rolled again.
pub const WATCH_MOVE: f64 = 90.0;

/// Seconds before a watch stops answering that it is rolled again.
pub const WATCH_SPARE: f64 = 10.0;

/// Seconds between two scans of which animals to draw.
pub const SCAN: f32 = 0.5;

/// Metres from the origin of its frame past which an animal starts again in a new frame, so its
/// flat frame never leans far from the local vertical.
pub const DRIFT: f32 = 120.0;

/// Seconds a grazing animal crops one patch before it steps to the next.
pub const GRAZE_EVERY: f64 = 9.0;

/// Spacings of its herd from its place within which a grazing animal steps to a new patch.
pub const GRAZE_REACH: f32 = 0.6;

/// Pull per second of an animal towards its place in the herd: a share of the gap closed each
/// second.
pub const PULL: f32 = 0.8;

/// Metres from its place within which an animal of a still herd stands still.
pub const SETTLE: f32 = 0.4;

/// Rise of the alarm per second while a threat is near, a share of the gap to its level.
pub const ALARM_RISE: f32 = 4.0;

/// Share of its top speed an animal flees at.
pub const FLEE_SPEED: f32 = 0.8;

/// Metres an animal of no size scavengers circle a carcass at; one body length more for each.
pub const RING: f32 = 1.5;

/// Metres of the radius of the loop a flock of flyers of no size circles above its place; it grows
/// with body length, up to four times.
pub const SOAR_LOOP: f32 = 14.0;

/// Share of its top speed a circling flyer flies at.
pub const SOAR_SPEED: f32 = 0.35;

/// Emissive strength, in the linear units of Bevy's emissive colour, of a glow of full strength at
/// midnight.
pub const NIGHT_GLOW: f32 = 6.0;

/// Most scavengers drawn at one carcass.
pub const VISITORS: u8 = 3;

/// Seconds after the kill before the first scavenger comes, and between two of them.
pub const VISIT_DELAY: f64 = 25.0;

/// Longest step in seconds an animal takes in one frame.
pub const MAX_STEP: f32 = 0.1;

/// Share of its top speed an attacker stalks up to a robot at, past [`RUSH`] metres.
pub const APPROACH_SPEED: f32 = 0.5;

/// Metres past its own length a body with no attack stops short of a robot it walks up to.
pub const STAND_OFF: f32 = 2.0;

/// Share of the reach of its attack at which an attacker stops closing, so the blow lands.
pub const STRIKE_SHARE: f32 = 0.8;

/// Metres from a threat within which an attacker runs at its top speed instead of stalking.
pub const RUSH: f32 = 12.0;

/// Metres past its strike within which an animal that stands its ground goes for a threat.
pub const GUARD: f32 = 3.0;
