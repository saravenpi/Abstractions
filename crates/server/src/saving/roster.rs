//! Who is in, to log joins and leaves.

use abstractions_net::wire::PlayerInfo;

/// The players seen at the last look.
#[derive(Debug)]
pub struct Roster {
    /// Players by id, as the room listed them.
    known: Vec<PlayerInfo>,
    /// Seats of the server, for the count in the log.
    cap: u16,
}

impl Roster {
    /// Returns an empty roster of a server with `cap` seats.
    pub fn new(cap: u16) -> Self {
        Self {
            known: Vec::with_capacity(usize::from(cap)),
            cap,
        }
    }

    /// Takes the players in now and returns one log line per player who left or joined.
    pub fn update(&mut self, players: Vec<PlayerInfo>) -> Vec<String> {
        let count = format!("({}/{})", players.len(), self.cap);
        let gone = |a: &PlayerInfo, list: &[PlayerInfo]| !list.iter().any(|b| b.id == a.id);
        let left = self.known.iter().filter(|old| gone(old, &players));
        let mut lines: Vec<String> = left
            .map(|old| format!("{} left {count}", old.name))
            .collect();
        let joined = players.iter().filter(|new| gone(new, &self.known));
        lines.extend(joined.map(|new| format!("{} joined {count}", new.name)));
        self.known = players;
        lines
    }

    /// Returns true when nobody is in.
    pub fn is_empty(&self) -> bool {
        self.known.is_empty()
    }
}
