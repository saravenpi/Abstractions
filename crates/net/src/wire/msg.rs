//! The reliable messages between a player and a server.
//!
//! The enums are append-only: a new message goes at the end, so the numbers of the old ones
//! never change. `Status` must stay first on both sides, because a server answers it before it
//! checks the version.

use serde::{Deserialize, Serialize};

use super::act::{Act, Event};
use super::chest::{ChestAct, ChestEvent, DigOp, Shot, TreeFell};
use super::fleet::{ShipAct, ShipEvent};
use crate::chest::PlacedProp;
use crate::ids::BodyId;
use crate::leaves::Run;
use crate::log::{Change, Changes, Entry, PlayerId};

/// What a server says about itself to anyone who asks, before any check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// Name of the server.
    pub name: String,
    /// Players in the world now.
    pub players: u16,
    /// Most players the server takes.
    pub cap: u16,
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

/// What a server keeps of a player between sessions. The server stores all texts as they are
/// and never reads `ship`, `marks`, or `pack`; the game writes TOML there.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerRecord {
    /// Where the player was last, or empty when unknown.
    pub place: String,
    /// The player's ship, or empty when the player has none yet.
    pub ship: String,
    /// The player's saved places, or empty when there are none.
    pub marks: String,
    /// The player's carried pack, or empty when unknown.
    #[serde(default)]
    pub pack: String,
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
    /// The same player joined again from another game, which took the seat.
    Replaced,
    /// The server heard nothing from the player's game for too long.
    Silent,
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
    /// The home rule of the world, which picks its home planet from the seed.
    pub home: u32,
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
    /// The leaves explored on each body, as runs of leaf keys.
    pub explored: Vec<(BodyId, Vec<Run>)>,
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
    /// The player explored new leaves of a body, as runs of leaf keys (see
    /// [`crate::leaves`]).
    Explored {
        /// The body explored.
        body: BodyId,
        /// The leaves newly explored.
        runs: Vec<Run>,
    },
    /// The player sets their waypoint, or clears it with nothing.
    Waypoint(Option<Waypoint>),
    /// The player tells what happened in their hunt: see [`Act`].
    Hunt(Act),
    /// The player asks to board or leave a ship: see [`ShipAct`].
    Ship(ShipAct),
    /// The player places, breaks, or updates a chest or torch: see [`ChestAct`].
    Chest(ChestAct),
    /// The player felled a tree on a body.
    Tree(TreeFell),
    /// The player carved a terrain crater on a body.
    Dig(DigOp),
    /// The player fired a firearm shot.
    Shot(Shot),
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
    /// Players explored new leaves of a body, numbered as in [`ClientMsg::Explored`].
    Explored {
        /// The body explored.
        body: BodyId,
        /// The leaves newly explored.
        runs: Vec<Run>,
    },
    /// A player set their waypoint, or cleared it with nothing.
    Waypoint {
        /// The player whose waypoint it is.
        player: PlayerId,
        /// The waypoint, or nothing when cleared.
        waypoint: Option<Waypoint>,
    },
    /// What another player did in their hunt, or the state of the hunt after joining: see
    /// [`Event`].
    Hunt(Event),
    /// Who pilots or releases a ship in the session: see [`ShipEvent`].
    Ship(ShipEvent),
    /// What happened to a placed chest or torch: see [`ChestEvent`].
    Chest(ChestEvent),
    /// Placed chests and torches sent in chunks after joining.
    Chests(Vec<PlacedProp>),
    /// Another player felled a tree.
    Tree(TreeFell),
    /// Felled trees sent in chunks after joining.
    Trees(Vec<TreeFell>),
    /// Another player dug a terrain crater.
    Dig(DigOp),
    /// Terrain dig craters sent in chunks after joining.
    Digs(Vec<DigOp>),
    /// Another player fired a firearm shot.
    Shot {
        /// Player who fired the shot.
        by: PlayerId,
        /// Parameters of the shot.
        shot: Shot,
    },
}
