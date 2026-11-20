//! Tuning of the drawn animals: how far and how many are drawn, how often the herds are read, and
//! how animals notice robots.
//!
//! Distances that animals keep from a threat grow with body size and with how much the species is
//! hunted, as flight initiation distances do in birds and mammals (Blumstein 2006, "Developing an
//! evolutionary ecology of fear", Animal Behaviour 71, 389-399).

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

/// Metres along the ground of the search radius of a watch: [`REACH`] and [`SLACK`] plus the room
/// the robot walks before the watch is rolled again.
pub const WATCH_RADIUS: f64 = REACH + SLACK + WATCH_MOVE;

/// Metres the robot walks from the search point of a watch before it is rolled again.
pub const WATCH_MOVE: f64 = 90.0;

/// Seconds before a watch stops answering that it is rolled again.
pub const WATCH_SPARE: f64 = 10.0;

/// Seconds between two scans of which animals to draw.
pub const SCAN: f32 = 0.5;

/// Metres from the origin of its frame past which an animal starts again in a new frame, so its
/// flat frame never leans far from the local vertical.
pub const DRIFT: f32 = 120.0;

/// Pull per second of an animal towards its place in the herd: a share of the gap closed each
/// second.
pub const PULL: f32 = 0.8;

/// Metres from its place within which an animal of a still herd stands still.
pub const SETTLE: f32 = 0.4;

/// Metres an animal of no size notices a robot from; [`NOTICE_PER_METRE`] more per metre of body.
pub const NOTICE: f32 = 8.0;

/// Metres added to the notice distance per metre of body length.
pub const NOTICE_PER_METRE: f32 = 6.0;

/// Share of the notice distance under which a fully wary animal flees.
pub const FLEE_SHARE: f32 = 0.6;

/// Rise of the alarm per second while a threat is near, a share of the gap to its level.
pub const ALARM_RISE: f32 = 4.0;

/// Share of its top speed an animal flees at.
pub const FLEE_SPEED: f32 = 0.8;

/// Metres an animal of no size scavengers circle a carcass at; one body length more for each.
pub const RING: f32 = 1.5;

/// Most scavengers drawn at one carcass.
pub const VISITORS: u8 = 3;

/// Seconds after the kill before the first scavenger comes, and between two of them.
pub const VISIT_DELAY: f64 = 25.0;

/// Highest Froude number, `v^2 / (g * leg)`, a walker is driven at: a full gallop, past which its
/// feet could no longer keep up with its body (Alexander and Jayes 1983, "A dynamic similarity
/// hypothesis for the gaits of quadrupedal mammals", Journal of Zoology 201, 135-152).
pub const FROUDE_TOP: f32 = 3.5;

/// Longest step in seconds an animal takes in one frame.
pub const MAX_STEP: f32 = 0.1;
