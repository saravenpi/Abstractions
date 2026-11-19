//! Running a choice of the menu of a right click on what was under the pointer.

use bevy::clipboard::Clipboard;
use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::{Choice, ContextMenu, Under};
use crate::computer::{Dot, MapView, Target, aimed};
use crate::map::geo::{coords, copy_near, to_chart};
use crate::map::level::{Level, framed};
use crate::map::roam::{focused, middle};
use crate::map::side::Do;
use crate::map::side_act::Desk;
use crate::map::waypoint::{WaypointChanged, Waypoints, place};
use crate::menus::{MarkerKind, MarkerPlace};
use crate::universe::BodyId;

/// What the choices of the menu change.
#[derive(SystemParam)]
pub struct Hands<'w, 's> {
    /// The favourites, the course, the level and the flat map, as the side panel changes them.
    desk: Desk<'w, 's>,
    /// The chart.
    view: ResMut<'w, MapView>,
    /// The waypoints and the news of a change for the server.
    waypoints: (ResMut<'w, Waypoints>, MessageWriter<'w, WaypointChanged>),
    /// The clipboard, where the window has one.
    clipboard: Option<ResMut<'w, Clipboard>>,
}

impl Hands<'_, '_> {
    /// Runs `choice` on `under`.
    fn run(&mut self, choice: Choice, under: Under) {
        match choice {
            Choice::Course => self.course(under),
            Choice::Focus => self.focus(under),
            Choice::Waypoint | Choice::Unwaypoint => self.waypoint(choice, under),
            Choice::Favourite | Choice::Rename | Choice::Delete => self.favourite(choice, under),
            Choice::Copy | Choice::Centre => self.ground(choice, under),
        }
    }

    /// Sets the course to the place, or to the planet the marker is on.
    fn course(&mut self, under: Under) {
        let target = match under {
            Under::Place(target) => Some(target),
            Under::Mine(body, _) | Under::Player(_, body) => Some(Target::Body(body)),
            Under::Favourite(index) => return self.desk.favourite(index, Do::Course),
            _ => None,
        };
        self.desk.destination.0 = target.or(self.desk.destination.0);
    }

    /// Eases the flat map of `body` onto `dir` when it is shown, or opens it looking there.
    fn look(&mut self, body: BodyId, dir: DVec3) {
        let (level, surface) = &mut self.desk.view;
        if **level != Level::Planet(body) {
            return self.desk.look(body, dir);
        }
        let at = copy_near(to_chart(dir), surface.lens.centre.x);
        surface.goal = focused(at, surface.goal.zoom);
    }

    /// Goes into the place, or focuses the flat map on the waypoint, the favourite or the
    /// player.
    fn focus(&mut self, under: Under) {
        match under {
            Under::Place(target) => {
                let board = &self.desk.board;
                let row = board.lines.iter().position(|line| line.target == target);
                let lens = row.and_then(|row| aimed(board, &self.view, Dot::Row(row)));
                self.view.goal = lens.unwrap_or(self.view.goal);
                self.view.into = Some(target);
            }
            Under::Mine(body, dir) => self.look(body, dir),
            Under::Favourite(index) => {
                if let Some(kept) = self.desk.favourites.0.get(index).cloned() {
                    self.look(kept.body, DVec3::from_array(kept.dir));
                }
            }
            Under::Player(index, body) => {
                let friends = self.desk.world.0.0.iter();
                let mut friends = friends.filter(|marker| marker.kind == MarkerKind::Friend);
                match friends.nth(index).map(|marker| marker.place) {
                    Some(MarkerPlace::Body(_, at)) => self.look(body, at),
                    _ => self.desk.player(index),
                }
            }
            _ => {}
        }
    }

    /// Sets the waypoint of this player at the place, or clears it.
    fn waypoint(&mut self, choice: Choice, under: Under) {
        let (waypoints, changes) = &mut self.waypoints;
        let change = match (choice, under) {
            (Choice::Unwaypoint, _) => waypoints.mine.take().map(|_| WaypointChanged(None)),
            (_, Under::Ground(body, dir)) => place(waypoints, body, dir),
            (_, Under::Place(Target::Body(body))) => place(waypoints, body, middle()),
            _ => None,
        };
        if let Some(change) = change {
            changes.write(change);
        }
    }

    /// Saves the place as a favourite, or renames or deletes the favourite.
    fn favourite(&mut self, choice: Choice, under: Under) {
        match (choice, under) {
            (Choice::Rename, Under::Favourite(index)) => self.desk.favourite(index, Do::Rename),
            (Choice::Delete, Under::Favourite(index)) => self.desk.favourite(index, Do::Delete),
            (Choice::Favourite, Under::Ground(body, dir) | Under::Mine(body, dir)) => {
                self.desk.keep(body, dir)
            }
            (Choice::Favourite, Under::Place(Target::Body(body))) => self.desk.keep(body, middle()),
            _ => {}
        }
    }

    /// Copies the coordinates of the place, or centres the map on it.
    fn ground(&mut self, choice: Choice, under: Under) {
        let surface = &mut self.desk.view.1;
        match (choice, under) {
            (Choice::Copy, Under::Ground(_, dir)) => {
                let copied = self
                    .clipboard
                    .as_mut()
                    .map(|clipboard| clipboard.set_text(coords(dir)));
                if let Some(Err(error)) = copied {
                    warn!("could not copy the coordinates: {error}");
                }
            }
            (Choice::Centre, Under::Ground(_, dir)) => {
                surface.goal.centre = copy_near(to_chart(dir), surface.goal.centre.x);
                surface.goal = framed(surface.goal);
            }
            (Choice::Centre, Under::Chart(at)) => {
                self.view.goal = crate::computer::Lens {
                    centre: at,
                    ..self.view.goal
                }
                .clamped();
            }
            _ => {}
        }
    }
}

/// Runs the choice clicked in the menu and closes it.
pub(super) fn on_choose(
    event: On<Activate>,
    lines: Query<&Choice>,
    mut menu: ResMut<ContextMenu>,
    mut hands: Hands,
) {
    let Ok(choice) = lines.get(event.entity) else {
        return;
    };
    if let Some(opened) = menu.0.take() {
        hands.run(*choice, opened.under);
    }
}
