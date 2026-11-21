//! Moving between the levels: showing a level, going into a place once the map eased onto it,
//! and going back.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{DIVE_TIME, Look, Place, Stop, Trail};
use crate::computer::{Board, Lens, MapView, Tab, Target};
use crate::map::level::Level;
use crate::map::surface::Surface;
use crate::space::Location;

/// What showing a level changes.
#[derive(SystemParam)]
pub struct Nav<'w> {
    /// The level shown.
    level: ResMut<'w, Level>,
    /// The tab of the chart and what it charts.
    board: ResMut<'w, Board>,
    /// The chart.
    view: ResMut<'w, MapView>,
    /// The flat map of a planet.
    surface: ResMut<'w, Surface>,
    /// The levels shown before.
    trail: ResMut<'w, Trail>,
    /// Where the player is, whose star a chart with nothing inside shows.
    location: Option<Res<'w, Location>>,
}

impl Nav<'_> {
    /// Returns the level shown now and how it is looked at.
    pub fn here(&self) -> Stop {
        let ship = self
            .location
            .as_ref()
            .map(|at| at.body.star)
            .unwrap_or_default();
        let chart = (self.board.tab, self.board.inside);
        let place = Place::shown(*self.level, chart, ship);
        let lens = match *self.level {
            Level::Chart => Look::Chart(self.view.goal),
            Level::Planet(_) => Look::Planet(self.surface.goal),
        };
        Stop {
            place,
            lens: Some(lens),
        }
    }

    /// Shows the chart of `tab` on `inside` through `lens`, with nothing pointed at or picked.
    fn chart(&mut self, (tab, inside): (Tab, Option<Target>), lens: Lens, same: bool) {
        if *self.level != Level::Chart {
            *self.level = Level::Chart;
        }
        if !same {
            self.board.open(tab, inside);
        }
        let view = &mut *self.view;
        (view.tab, view.lens, view.goal, view.cursor) = (tab, lens, lens, 0);
        (view.hover, view.clicked, view.into) = (None, None, None);
        (view.picked, view.marked) = (false, false);
    }

    /// Shows the level of `stop`, looked at as it was, or as it opens.
    pub fn go(&mut self, stop: Stop) {
        let same = self.here().place == stop.place;
        let (lens, sight) = match stop.lens {
            Some(Look::Chart(lens)) => (lens, None),
            Some(Look::Planet(sight)) => (Lens::default(), Some(sight)),
            None => (Lens::default(), None),
        };
        match stop.place {
            Place::Planet(_) if same => self.surface.goal = sight.unwrap_or(self.surface.goal),
            Place::Planet(body) => {
                self.surface.again = sight;
                *self.level = Level::Planet(body);
            }
            Place::Universe => self.chart((Tab::Universe, None), lens, same),
            Place::Galaxy(galaxy) => {
                self.chart((Tab::Galaxy, Some(Target::Galaxy(galaxy))), lens, same)
            }
            Place::System(star) => self.chart((Tab::System, Some(Target::Star(star))), lens, same),
        }
    }

    /// Goes into `place` and keeps the level shown now to come back to. Nothing happens when
    /// it is the level shown.
    pub fn enter(&mut self, place: Place) {
        let here = self.here();
        if here.place != place {
            self.trail.stops.push(here);
            self.go(Stop { place, lens: None });
        }
    }

    /// Goes back to the level shown before, or else to the level around this one.
    pub fn back(&mut self) {
        let here = self.here().place;
        if let Some(stop) = self.trail.back(here) {
            self.trail.pending = None;
            self.go(stop);
        }
    }
}

/// Takes a place the map was asked to go into, then goes into it once the chart has eased
/// onto it or [`DIVE_TIME`] has passed. A place that is the level shown is only eased onto.
pub(super) fn descend(time: Res<Time>, mut nav: Nav) {
    if let Some(target) = nav.view.into.take() {
        let place = Place::of(target);
        if place != nav.here().place {
            nav.trail.pending = Some((place, DIVE_TIME));
        }
    }
    let Some((place, left)) = nav.trail.pending else {
        return;
    };
    let left = left - time.delta_secs();
    let eased = nav.view.lens == nav.view.goal;
    nav.trail.pending = (left > 0.0 && !eased).then_some((place, left));
    if nav.trail.pending.is_none() {
        nav.enter(place);
    }
}
