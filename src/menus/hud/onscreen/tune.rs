//! The numbers of the markers over the world, each with its unit and its reason. The rules
//! themselves and their sources are in `handoff/specs/markers.md`.

/// Lengths of its own target, in spans, under which a marker is reduced: the target is ten
/// times its length away, about six degrees wide, and is its own sign from there.
pub const REDUCE_SPANS: f64 = 10.0;

/// Lengths of its own target, in spans, under which a marker is hidden: the target fills the
/// view and the icon would only cover it.
pub const HIDE_SPANS: f64 = 4.0;

/// Ratio of the outer limit of a fade to its inner limit: a marker fades over the outer quarter
/// of the distance, never at once.
pub const BAND: f64 = 1.25;

/// Ratio by which the limits of a fade move out once it has run to its end, so a player who
/// stands on a limit sees no flicker: the marker comes back a fifth farther than it left.
pub const STICK: f64 = 1.2;

/// Metres past which the marker of a pin nobody follows fades out on foot or swimming: two
/// minutes at a sprint of 8.5 m/s (`player/walk.rs`). Farther, it is noise over the world.
pub const FAR_FOOT: f64 = 1000.0;

/// Metres past which the marker of a pin nobody follows fades out while gliding: two minutes at
/// about 45 m/s.
pub const FAR_GLIDE: f64 = 5000.0;

/// Metres past which the marker of a pin nobody follows fades out in the ship, landed or in
/// flight: two minutes at 140 m/s, the top speed 400 m over the ground (`ship/cruise.rs`).
pub const FAR_SHIP: f64 = 15000.0;

/// Metres from a pin the player follows under which it has been reached and its marker hides.
pub const ARRIVE: f64 = 8.0;

/// Length in metres of the container the home marker stands on.
pub const HOME_SPAN: f64 = 6.0;

/// Length in metres of the ship.
pub const SHIP_SPAN: f64 = 8.0;

/// Height in metres of a player, and the room a pin with nothing built on it is given.
pub const BODY_SPAN: f64 = 2.0;

/// How many times farther a target counts as near while gliding: the ground passes faster.
pub const GLIDE_REACH: f64 = 2.0;

/// How many times farther a target counts as near while the ship flies: at its speed the last
/// hundreds of metres are the approach, flown by eye on the pad and its beacon.
pub const FLIGHT_REACH: f64 = 5.0;

/// Degrees from the middle of the view within which a marker is at its faintest, and past which
/// it is whole: it never covers what the player aims at.
pub const CENTRE: (f32, f32) = (5.0, 12.0);

/// Opacity of a marker in the middle of the view, from 0 to 1.
pub const CENTRE_ALPHA: f32 = 0.35;

/// Opacity of the icon of a reduced marker, from 0 to 1.
pub const REDUCED_ALPHA: f32 = 0.75;

/// Metres above the middle of a friend their marker stands at: over the head.
pub const FRIEND_LIFT: f32 = 1.5;

/// Pixels from the centre of an icon to the centre of its arrow.
pub const ARROW_REACH: f32 = 22.0;

/// Pixels a marker keeps free on each side of its icon and above it: half its room across, the
/// arrow above.
pub const HALF: (f32, f32) = (48.0, 36.0);

/// Share of its opacity the icon of a marker keeps while the planet stands between the eye and
/// its place: hollow, at its true place, its words saying why.
pub const HIDDEN_ALPHA: f32 = 0.5;

/// What the words of a marker add while the planet stands between the eye and its place.
pub const HIDDEN_WORDS: &str = ", behind the planet";
