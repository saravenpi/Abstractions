//! The table rows of each tab, built from the universe and the atlas.

use bevy::math::DVec3;

use super::atlas::Atlas;
use super::detail::{brief, charted};
use super::format::{distance, gravity};
use super::home::homes;
use super::route::{HOME_LABEL, Target};
use crate::space::UniPos;
use crate::universe::{GalaxyId, GalaxySpec, PlanetSpec, StarId, StarSpec, Universe};

/// One selectable row of a tab.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// Name of the thing.
    pub title: String,
    /// Its facts, starting with the distance.
    pub detail: String,
    /// What selecting the row sets as the destination.
    pub target: Target,
    /// Distance from the ship in metres.
    pub distance: f64,
    /// Whether the player has been there.
    pub visited: bool,
    /// Where its centre is in the universe, for the map.
    pub at: UniPos,
}

/// What ends the detail of a row whose target was visited.
const VISITED: &str = " - visited";

impl Line {
    /// Returns the facts of the row alone: the detail without its leading distance and without
    /// the visited mark.
    pub fn facts(&self) -> &str {
        let rest = self.detail.split_once(" - ").map_or("", |(_, rest)| rest);
        rest.strip_suffix(VISITED).unwrap_or(rest)
    }
}

/// Orders rows by distance, nearest first.
fn nearest_first(mut lines: Vec<Line>) -> Vec<Line> {
    lines.sort_by(|a, b| a.distance.total_cmp(&b.distance));
    lines
}

/// What the rows are built from.
#[derive(Debug, Clone, Copy)]
pub struct View<'a> {
    /// The universe the specs are generated from.
    pub universe: &'a Universe,
    /// What the player knows.
    pub atlas: &'a Atlas,
    /// Where the ship is.
    pub ship: UniPos,
    /// Universe time in seconds, which places the planets on their orbits.
    pub time: f64,
}

impl View<'_> {
    /// Builds a row for something whose centre is `at` and whose surface is `radius` metres out.
    /// The home star and the home galaxy say so after their name, as the home planet does.
    fn line(&self, title: &str, target: Target, place: (UniPos, f64), facts: String) -> Line {
        let gap = (place.0.distance(self.ship) - place.1).max(0.0);
        let visited = self.atlas.visited(target);
        let mark = if visited { VISITED } else { "" };
        let ours = homes(self.universe)[1..].contains(&Some(target));
        let label = if ours {
            format!(" {HOME_LABEL}")
        } else {
            String::new()
        };
        Line {
            title: format!("{title}{label}"),
            detail: format!("{} - {facts}{mark}", distance(gap)),
            target,
            distance: gap,
            visited,
            at: place.0,
        }
    }

    /// Returns the row of a planet whose star is at `origin`. Its class and its temperature
    /// show only once the system is visited.
    fn planet(&self, origin: UniPos, planet: &PlanetSpec) -> Line {
        let target = Target::Body(planet.id);
        let record = charted(self.atlas, target).then_some(&planet.record);
        let facts = format!(
            "{} - radius {} - {}",
            brief(record),
            distance(planet.radius),
            gravity(planet.gravity)
        );
        let at = origin + planet.position(self.time);
        let title = target.name(self.universe);
        self.line(&title, target, (at, planet.radius), facts)
    }

    /// Returns the row of a star at `at`. The planet count shows only once the system is visited.
    fn star(&self, at: UniPos, star: &StarSpec) -> Line {
        let target = Target::Star(star.id);
        let planets = match self.atlas.visited(target) {
            true => format!("{} planets", self.universe.system(star.id).planets.len()),
            false => "planets unknown".to_string(),
        };
        let facts = format!("class {:?} - {planets}", star.class);
        self.line(&star.name, target, (at, star.radius), facts)
    }

    /// Returns the rows of the system tab: the star, then its planets ordered outward, each
    /// followed by its moons ordered outward once the system is visited.
    pub fn system(&self, star: StarId) -> Vec<Line> {
        let system = self.universe.system(star);
        let origin = self
            .universe
            .star_position(system.star.id)
            .unwrap_or_default();
        let charted = self.atlas.visited(Target::Star(system.star.id));
        let mut lines = vec![self.star(origin, &system.star)];
        for planet in &system.planets {
            lines.push(self.planet(origin, planet));
            let moons = match charted && planet.moons > 0 {
                true => self.universe.moons(planet.id),
                false => Vec::new(),
            };
            lines.extend(moons.iter().map(|moon| self.planet(origin, moon)));
        }
        lines
    }

    /// Returns the rows of the galaxy tab: the stars of `galaxy` among `known`, nearest first.
    /// The home star is always one of them.
    pub fn galaxy(&self, galaxy: GalaxyId, known: impl Iterator<Item = StarId>) -> Vec<Line> {
        let Some(spec) = self.universe.galaxy(galaxy) else {
            return Vec::new();
        };
        let home = self.universe.home().star;
        let known = known.filter(|id| *id != home).chain([home]);
        let stars = known
            .filter(|id| id.galaxy == galaxy)
            .filter_map(|id| self.universe.star(id));
        nearest_first(
            stars
                .map(|star| self.star(spec.locate(star.pos), &star))
                .collect(),
        )
    }

    /// Returns the rows of the universe tab: every known galaxy, nearest first.
    pub fn universe(&self) -> Vec<Line> {
        let known = self.atlas.known_galaxies(self.universe);
        let row = |galaxy: &GalaxySpec| {
            let facts = format!("{:?}", galaxy.kind).to_lowercase();
            let centre = (galaxy.locate(DVec3::ZERO), 0.0);
            self.line(&galaxy.name, Target::Galaxy(galaxy.id), centre, facts)
        };
        nearest_first(known.iter().map(row).collect())
    }
}
