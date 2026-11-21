//! Checking the place a server keeps of a player, so a damaged one never reaches a game.
//!
//! The server has no generator, so it cannot tell where the ground of a planet is. It checks
//! what it can: the text reads as a place, its numbers are finite, its direction has a length
//! and its height is within [`LOWEST`] and [`HIGHEST`]. A height out of that range becomes
//! [`LOWEST`], which is under any ground: the game stands a body saved under the ground on the
//! ground in the same direction. A place that cannot be read at all is forgotten, and the game
//! then starts the player at the base. The game itself checks that the body exists.

use serde::{Deserialize, Serialize};

use super::{Member, World};
use crate::ids::BodyId;

/// Lowest height above sea level a stored place may have, in metres: deeper than any ground.
/// A place at this height means the ground in its direction.
pub const LOWEST: f64 = -20_000.0;

/// Highest height above sea level a stored place may have, in metres: above the air of any
/// planet a body can be saved on.
pub const HIGHEST: f64 = 1_000_000.0;

/// Shortest direction that still points somewhere.
const DIR_MIN: f64 = 1.0e-6;

/// Most characters of a bad place quoted in the log.
const QUOTED: usize = 160;

/// A place as the game writes it: a body, a direction from its centre and a height in metres
/// above its sea level.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Stored {
    /// Direction from the centre of the body.
    dir: [f64; 3],
    /// Height above sea level in metres.
    altitude: f64,
    /// The body.
    body: BodyId,
}

/// Returns the text that must replace the stored place `text`, or nothing when it may stay. An
/// empty text is an unknown place and stays. A text that does not read as a place, or whose
/// direction is not finite or has no length, is replaced by the empty text. A height that is
/// not a number or out of range is replaced by [`LOWEST`], the direction and the body kept.
pub fn repaired(text: &str) -> Option<String> {
    if text.trim().is_empty() {
        return None;
    }
    let Ok(stored) = toml::from_str::<Stored>(text) else {
        return Some(String::new());
    };
    let length = stored
        .dir
        .iter()
        .map(|part| part * part)
        .sum::<f64>()
        .sqrt();
    if !(length.is_finite() && length > DIR_MIN) {
        return Some(String::new());
    }
    if (LOWEST..=HIGHEST).contains(&stored.altitude) {
        return None;
    }
    let grounded = Stored {
        altitude: LOWEST,
        ..stored
    };
    Some(toml::to_string(&grounded).unwrap_or_default())
}

/// Repairs the place of `member` and returns the line that says so for the log, or nothing
/// when the place may stay.
pub fn mend(member: &mut Member) -> Option<String> {
    let fixed = repaired(&member.place)?;
    let done = match fixed.is_empty() {
        true => "it is forgotten and the player starts at the base",
        false => "the player is put on the ground in the same direction",
    };
    let old: String = member
        .place
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let quoted: String = old.chars().take(QUOTED).collect();
    let line = format!(
        "The stored place of {} is not valid, {done}. It was: {quoted}",
        member.name
    );
    member.place = fixed;
    Some(line)
}

/// Repairs the place of every player of `world` and returns one line per repair for the log.
pub fn mend_all(world: &mut World) -> Vec<String> {
    world.players.values_mut().filter_map(mend).collect()
}
