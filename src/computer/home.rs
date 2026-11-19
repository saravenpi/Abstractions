//! Home: the planet a game starts on, its star and its galaxy. They are always known, the way
//! back to them is worked out from where the ship is, and nothing of it is stored.

use bevy::math::DVec3;

use super::locate::{here, ship_position, target_position};
use super::route::Target;
use super::rows::Line;
use super::sift::{Sort, arrange};
use super::tab::Tab;
use crate::space::{Frame, UniPos};
use crate::universe::Universe;

/// Returns the home planet, its star and its galaxy, in the order of what the ship is inside.
pub fn homes(universe: &Universe) -> [Option<Target>; 3] {
    let home = universe.home();
    [
        Some(Target::Body(home)),
        Some(Target::Star(home.star)),
        Some(Target::Galaxy(home.star.galaxy)),
    ]
}

/// Returns the next place on the way home for a ship inside `here`: the home galaxy from
/// another galaxy, the home star from another system of the home galaxy, and the home planet
/// from inside the home system.
pub fn toward(here: [Option<Target>; 3], universe: &Universe) -> Target {
    let home = universe.home();
    if here[2] != Some(Target::Galaxy(home.star.galaxy)) {
        Target::Galaxy(home.star.galaxy)
    } else if here[1] != Some(Target::Star(home.star)) {
        Target::Star(home.star)
    } else {
        Target::Body(home)
    }
}

/// Returns the tab whose table lists `target`.
pub fn tab_of(target: Target) -> Tab {
    match target {
        Target::Body(_) => Tab::System,
        Target::Star(_) => Tab::Galaxy,
        Target::Galaxy(_) => Tab::Universe,
    }
}

/// Returns the next place on the way home and where its centre is at `time` seconds, or
/// nothing for a ship that is on the home planet.
pub fn beacon(
    universe: &Universe,
    here: [Option<Target>; 3],
    time: f64,
) -> Option<(Target, UniPos)> {
    let target = toward(here, universe);
    let at = target_position(universe, target, time)?;
    (here[0] != Some(target)).then_some((target, at))
}

/// Returns the direction of the next place on the way home, in the axes of the systems, and
/// its distance in metres, seen from the point `at` of `frame` at `time` seconds. Nothing on
/// the home planet.
pub fn homeward(universe: &Universe, frame: Frame, at: DVec3, time: f64) -> Option<(DVec3, f64)> {
    let (_, to) = beacon(universe, here(universe, frame, at), time)?;
    let offset = to - ship_position(universe, frame, at, time)?;
    Some((offset.try_normalize()?, offset.length()))
}

/// Returns the row pinned at the top of the table of `tab` for a ship inside `here`: the home
/// galaxy on the universe tab away from it, the home star on the galaxy tab away from the
/// home system, nothing on the system tab.
pub fn pinned(tab: Tab, here: [Option<Target>; 3], universe: &Universe) -> Option<Target> {
    let level = tab as usize;
    let home = homes(universe)[level];
    home.filter(|_| tab != Tab::System && here[level] != home)
}

/// Returns the rows as `arrange` orders and narrows them, with the row of `pin` first
/// whatever the order and the filter say, so the way home is never lost in a long list.
pub fn arranged(mut lines: Vec<Line>, pin: Option<Target>, filter: &str, sort: Sort) -> Vec<Line> {
    let at = pin.and_then(|pin| lines.iter().position(|line| line.target == pin));
    let first = at.map(|at| lines.remove(at));
    let mut rows = arrange(lines, filter, sort);
    rows.splice(0..0, first);
    rows
}
