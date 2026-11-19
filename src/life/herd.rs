//! Herds: groups of animals that are pure functions of the life of a planet, its seed and universe time.
//!
//! Every player computes the same herds at the same time with no network. The sphere is cut into
//! cells of about 256 m (see `cells`); each cell holds herds rolled from the census density at its
//! centre times its area. A herd has a home in its cell and walks a route of waypoints inside its
//! home range (see `route`). Herd centres are unit directions; members are offsets in metres in the
//! tangent plane, x east and y north, north being the spin axis, world +Y.

mod cells;
#[cfg(test)]
pub(super) mod fixtures;
mod roll;
mod route;
mod sphere;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cost;

use bevy::math::{DVec3, Vec2};

use super::Life;
use super::census::Site;
use crate::universe::hash::{Rng, child};
use cells::Grid;
use roll::Roam;

/// What a herd does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HerdState {
    /// Feeds at a waypoint.
    Graze,
    /// Walks between waypoints.
    Travel,
    /// Drinks at a waypoint by the water.
    Drink,
    /// Stays still at a waypoint outside its active hours.
    Rest,
    /// Sleeps at a waypoint deep in its inactive hours.
    Sleep,
}

/// One herd at one time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Herd {
    /// Stable identity.
    pub id: u64,
    /// Index of its species in [`Life::species`].
    pub species: u16,
    /// Members, young included.
    pub count: u8,
    /// Young among the members.
    pub young: u8,
    /// Unit direction of its centre.
    pub centre: DVec3,
    /// Unit tangent at the centre it faces.
    pub heading: DVec3,
    /// Speed of the centre in metres per second.
    pub speed: f32,
    /// What it does.
    pub state: HerdState,
    /// Unit direction of the centre of its home range.
    pub home: DVec3,
    /// Usual distance between members in metres, from the size of its species.
    pub spacing: f32,
}

/// One member of a herd at one time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Member {
    /// Offset from the herd centre in metres in the tangent plane, x east and y north.
    pub offset: Vec2,
    /// True for the young.
    pub young: bool,
    /// Phase of its gait and idle motion, 0 to 1.
    pub phase: f32,
}

/// Where and when to look for herds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Search {
    /// Unit direction looked around.
    pub at: DVec3,
    /// Distance in metres along the ground within which herd centres are returned.
    pub radius: f64,
    /// Sea level radius of the planet in metres.
    pub planet: f64,
    /// Universe time in seconds.
    pub time: f64,
    /// The robot's wake point on the home planet, as a unit direction, whose herd is always there.
    pub pin: Option<DVec3>,
}

/// Returns every herd of `life` whose centre is within the search radius at the search time, in a stable
/// order. `sample` gives the site at a unit direction; it is called at most a few hundred times.
pub fn near(life: &Life, sample: &impl Fn(DVec3) -> Site, search: &Search) -> Vec<Herd> {
    let grid = Grid::of(search.planet);
    let widest = roll::widest(life);
    let reach = search.radius + widest + grid.side();
    let pinned = search
        .pin
        .filter(|pin| sphere::arc(*pin, search.at) * search.planet <= search.radius);
    grid.around(search.at, reach)
        .into_iter()
        .filter(|c| sphere::arc(grid.spot(*c, 0.5, 0.5), search.at) * search.planet <= reach)
        .flat_map(|c| roll::in_cell(life, &grid, c, &sample(grid.spot(c, 0.5, 0.5))))
        .chain(pinned.and_then(|pin| roll::pinned(life, pin)))
        .filter(|r| {
            sphere::arc(r.home, search.at) * search.planet <= search.radius + r.reach + grid.side()
        })
        .filter_map(|r| herd(life, &r, sample, search))
        .filter(|h| sphere::arc(h.centre, search.at) * search.planet <= search.radius)
        .collect()
}

/// Returns `roam` at the search time, or nothing when its species is missing.
fn herd(
    life: &Life,
    roam: &Roam,
    sample: &impl Fn(DVec3) -> Site,
    search: &Search,
) -> Option<Herd> {
    let species = life.species.get(usize::from(roam.species))?;
    let pose = route::pose(roam, species, search.time, search.planet, sample);
    Some(Herd {
        id: roam.id,
        species: roam.species,
        count: roam.count,
        young: roam.young,
        centre: pose.centre,
        heading: pose.heading,
        speed: pose.speed,
        state: pose.state,
        home: roam.home,
        spacing: (0.6 * species.history.mass.cbrt()).max(0.1),
    })
}

/// Returns member `slot` of `herd` at universe `time`: the young nearest the centre, each member at its
/// own seeded place on a sunflower spiral, drifting slowly around it.
pub fn member(herd: &Herd, slot: u8, time: f64) -> Member {
    let mut rng = Rng::new(child(herd.id, u64::from(slot)));
    let rank = f64::from(slot) + 0.5;
    let spacing = f64::from(herd.spacing);
    let angle = rank * 2.399_963 + Rng::new(herd.id).unit() * std::f64::consts::TAU;
    let ring = spacing * rank.sqrt();
    let jitter = (rng.range(-0.35, 0.35), rng.range(-0.35, 0.35));
    let (rate, shift) = (rng.range(0.02, 0.06), rng.range(0.0, std::f64::consts::TAU));
    let drift = 0.3 * spacing;
    let x = ring * angle.cos() + jitter.0 * spacing + drift * (rate * time + shift).sin();
    let y = ring * angle.sin() + jitter.1 * spacing + drift * (rate * 0.8 * time + shift).cos();
    Member {
        offset: Vec2::new(x as f32, y as f32),
        young: slot < herd.young,
        phase: rng.unit() as f32,
    }
}
