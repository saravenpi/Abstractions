//! The reliable messages between a player and a server.
//!
//! The enums are append-only: a new message goes at the end, so the numbers of the old ones
//! never change. `Status` must stay first on both sides, because a server answers it before it
//! checks the version.

use serde::{Deserialize, Serialize};

use crate::ids::BodyId;
use crate::log::{Change, Changes, Entry, PlayerId};

/// What a server says about itself to anyone who asks, before any check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// Name of the server.
    pub name: String,
    /// Players in the world now.
    pub players: u8,
    /// Most players the server takes.
    pub cap: u8,
    /// Version of the server program, such as `0.9.0`.
    pub version: String,
    /// Protocol the server speaks: compare it with [`super::PROTOCOL`].
    pub protocol: u16,
}

/// A player in the world, as the others see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// Id of the player in this session.
    pub id: PlayerId,
    /// Name of the player, already cleaned by [`super::clean_name`].
    pub name: String,
}

/// What a server keeps of a player between sessions. The server stores both texts as they are
/// and never reads them; the game writes TOML there.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerRecord {
    /// Where the player was last, or empty when unknown.
    pub place: String,
    /// The player's ship, or empty when the player has none yet.
    pub ship: String,
    /// The player's saved places, or empty when there are none.
    pub marks: String,
}

/// A point a player marks for the others: a direction from the centre of a body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    /// The body the point is on.
    pub body: BodyId,
    /// Unit direction from the body centre to the point, in the body's turning frame.
    pub dir: [f32; 3],
    /// Name of the point, or empty when it has none.
    pub name: String,
}

/// Why a server refuses a player or sends one away. New reasons are added at the end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Reason {
    /// The password is missing or wrong.
    WrongPassword,
    /// The server holds as many players as it takes.
    Full,
    /// The player's game is older than the server.
    Outdated,
    /// The player's game is newer than the server.
    Newer,
    /// The player sent more than the server allows.
    Flooding,
    /// The server is closing.
    Closing,
}

/// How a player asks to join.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    /// Protocol the player speaks.
    pub protocol: u16,
    /// Name the player asks for. The server cleans it with [`super::clean_name`].
    pub name: String,
    /// Stable identity of the player: the server finds the player's record by it.
    pub key: [u8; 32],
    /// Password of the server, when the player gives one.
    pub password: Option<String>,
}

/// Everything a player needs to enter the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Welcome {
    /// Id the server gives the player for this session.
    pub you: PlayerId,
    /// Seed of the universe.
    pub seed: u64,
    /// Universe time now, in seconds.
    pub clock: f64,
    /// Every change made to the world so far.
    pub changes: Changes,
    /// The atlas all players share, as text the server stores and never reads.
    pub atlas: String,
    /// What the server kept of this player, or nothing on a first visit.
    pub record: Option<PlayerRecord>,
    /// The players already in the world, without the one who joins.
    pub players: Vec<PlayerInfo>,
    /// The cells explored on each body, as bitsets of 64 cells a word, cell 0 in bit 0 of word 0.
    pub explored: Vec<(BodyId, Vec<u64>)>,
    /// The live waypoint of each player already in the world.
    pub waypoints: Vec<(PlayerId, Waypoint)>,
}

/// A message from a player to a server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMsg {
    /// Asks what the server is. Answered with [`ServerMsg::Status`] before any check.
    Status,
    /// Asks to join.
    Hello(Hello),
    /// The player changed the world. The server stamps the time and the player itself.
    Change(Change),
    /// The player's place and ship, for the server to keep.
    Record(PlayerRecord),
    /// Asks for the universe time, to measure the delay.
    Ping {
        /// Number of the ping, sent back as it is.
        id: u32,
        /// The player's own clock when the ping left, in seconds, sent back as it is.
        sent: f64,
    },
    /// The player leaves.
    Bye,
    /// The player explored new cells of a body. A cell is `row * 512 + column` on the grid of
    /// [`super::EXPLORED_ROWS`] by [`super::EXPLORED_COLUMNS`].
    Explored {
        /// The body explored.
        body: BodyId,
        /// The cells newly explored.
        cells: Vec<u32>,
    },
    /// The player sets their waypoint, or clears it with nothing.
    Waypoint(Option<Waypoint>),
}

/// A message from a server to a player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMsg {
    /// What the server is.
    Status(Status),
    /// The player is in.
    Welcome(Welcome),
    /// The player is not let in.
    Refused(Reason),
    /// Another player came in.
    Joined(PlayerInfo),
    /// Another player left.
    Left(PlayerId),
    /// Someone changed the world: a new entry of the log.
    Change(Entry),
    /// The answer to a ping.
    Pong {
        /// Number of the ping.
        id: u32,
        /// The player's clock the ping carried.
        sent: f64,
        /// Universe time when the server answered, in seconds.
        clock: f64,
    },
    /// The player is sent away.
    Kick(Reason),
    /// Players explored new cells of a body, numbered as in [`ClientMsg::Explored`].
    Explored {
        /// The body explored.
        body: BodyId,
        /// The cells newly explored.
        cells: Vec<u32>,
    },
    /// A player set their waypoint, or cleared it with nothing.
    Waypoint {
        /// The player whose waypoint it is.
        player: PlayerId,
        /// The waypoint, or nothing when cleared.
        waypoint: Option<Waypoint>,
    },
}
