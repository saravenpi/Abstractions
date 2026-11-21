//! Go: the one command that sends the ship to a place. It sets the course, hands it to the
//! autopilot and closes the map, or stops the route that is flown. The Go key, Enter, the
//! button of the card and the home key all end here.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::course::{Press, press};
use super::locate::here;
use super::open::Computer;
use super::route::{Cancel, Destination, Engage, Homeward, RouteStatus, Target};
use crate::space::{Anchor, Frame, Location, Travel};
use crate::universe::{BodyId, Universe};

/// What a press of Go reads and writes.
#[derive(SystemParam)]
pub struct Go<'w, 's> {
    destination: ResMut<'w, Destination>,
    route: Res<'w, RouteStatus>,
    location: Option<Res<'w, Location>>,
    world: Option<(Res<'w, Universe>, Res<'w, Anchor>)>,
    state: Option<Res<'w, State<Computer>>>,
    next: Option<ResMut<'w, NextState<Computer>>>,
    commands: Commands<'w, 's>,
}

impl Go<'_, '_> {
    /// Returns whether the player is at `target`: on its planet, or inside its system or galaxy.
    fn at(&self, target: Target) -> bool {
        self.world.as_ref().is_some_and(|(universe, anchor)| {
            here(universe, anchor.frame, anchor.origin).contains(&Some(target))
        })
    }

    /// Returns whether the player sits in the ship.
    fn seated(&self) -> bool {
        let seated = |location: &Location| location.travel == Travel::InShip;
        self.location.as_deref().is_some_and(seated)
    }

    /// Closes the map, if it is open, once the ship is on its way.
    fn close(&mut self) {
        let open = self
            .state
            .as_deref()
            .is_some_and(|s| *s.get() == Computer::Open);
        if let Some(next) = self.next.as_mut().filter(|_| open) {
            next.set(Computer::Closed);
        }
    }

    /// Presses Go on the place `pick`, or on the course when nothing is picked: sends the ship
    /// there, or stops the route that already flies there.
    pub fn press(&mut self, pick: Option<Target>) {
        let target = pick.or(self.destination.0);
        let can = self.seated() && !target.is_some_and(|target| self.at(target));
        match press(pick, self.destination.0, *self.route, can) {
            Press::Go(target) => {
                self.destination.0 = Some(target);
                self.commands.trigger(Engage(target));
                self.close();
            }
            Press::Stop => self.commands.trigger(Cancel),
            Press::Nothing => {}
        }
    }

    /// Returns whether the player is on the home planet `home` or above it in its system, where
    /// the world streams it: the ship can cruise there.
    fn over(&self, home: BodyId) -> bool {
        let streamed = self.location.as_deref().is_some_and(|at| at.body == home);
        let frame = self.world.as_ref().map(|(_, anchor)| anchor.frame);
        let near = match frame {
            Some(Frame::Surface(body)) => body == home,
            Some(Frame::System(star)) => star == home.star,
            _ => false,
        };
        streamed && near
    }

    /// Presses Go on the home planet: one command, whatever galaxy the ship is in. Seated on or
    /// above the home planet, it asks the ship to cruise to the base instead; away from it, the
    /// ship also cruises to the base once the route arrives.
    pub fn home(&mut self) {
        let Some(home) = self.world.as_ref().map(|(universe, _)| universe.home()) else {
            return;
        };
        let (seated, over) = (self.seated(), self.over(home));
        if !(seated && over) {
            self.press(Some(Target::Body(home)));
        }
        if seated {
            self.commands.trigger(Homeward(home));
        }
        if seated && over {
            self.close();
        }
    }
}
