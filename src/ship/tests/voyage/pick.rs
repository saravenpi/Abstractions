//! Picking a place to go to: the nearest other planet of the home system and the nearest other star.

use crate::ship::tests::rig::Game;
use crate::space::Clock;
use crate::universe::{BodyId, StarId, StarSpec, Universe};

/// Light-years around the home star in which another star is looked for.
const NEAR: f64 = 12.0;

/// Returns the planet of the home system nearest to the home planet now, itself excluded.
pub fn neighbour(game: &Game) -> BodyId {
    let (universe, time) = (game.get::<Universe>(), game.get::<Clock>().seconds);
    let home = universe.home();
    let system = universe.system(home.star);
    let from = universe.planet_position(home, time);
    let gap = |id: BodyId| universe.planet_position(id, time).distance(from);
    let others = system.planets.iter().map(|planet| planet.id);
    let nearest = others
        .filter(|id| *id != home)
        .min_by(|a, b| gap(*a).total_cmp(&gap(*b)));
    nearest.expect("another planet")
}

/// Returns the nearest other star of the home galaxy that has planets.
pub fn other_star(game: &Game) -> StarId {
    let universe = game.get::<Universe>();
    let home = universe.home();
    let at = universe.star(home.star).expect("the home star").pos;
    let stars = universe.stars_near(home.star.galaxy, at, NEAR);
    let fit =
        |star: &&StarSpec| star.id != home.star && !universe.system(star.id).planets.is_empty();
    stars.iter().find(fit).expect("another star").id
}
