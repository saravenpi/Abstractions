//! Where the other players are, as the newest of their records says: what the markers of the
//! players read.

use abstractions_net::wire;
use bevy::math::DVec3;

use super::robot::mode;
use super::track::Remotes;
use crate::avatar::Mode;
use crate::sim::PlayerId;

impl Remotes {
    /// Returns every other player with the wire frame and the position of its newest record:
    /// its ship while its robot sits in it, else its robot, else its ship.
    pub fn whereabouts(&self) -> impl Iterator<Item = (PlayerId, (wire::Frame, DVec3))> + '_ {
        let walkers = self.robots.iter().map(|(player, robot)| {
            let seated = mode(robot.latest.mode) == Mode::Seated;
            let ship = self.ships.get(player).filter(|_| seated);
            let at = ship.map_or((robot.frame, robot.latest.position), |ship| {
                (ship.frame, ship.latest.position)
            });
            (*player, at)
        });
        let alone = self
            .ships
            .iter()
            .filter(|(player, _)| !self.robots.contains_key(player));
        walkers.chain(alone.map(|(player, ship)| (*player, (ship.frame, ship.latest.position))))
    }

    /// Returns whether a record of `player` is kept.
    pub fn knows(&self, player: PlayerId) -> bool {
        self.robots.contains_key(&player) || self.ships.contains_key(&player)
    }
}
