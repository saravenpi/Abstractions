//! Computer tests: the reveal rules of the atlas and the atlas on disk.

use std::collections::HashSet;

use super::*;
use crate::universe::{GLY, GalaxyId, StarId, Universe};

/// A new atlas knows the home galaxy and the three galaxies nearest to it, and no other.
#[test]
fn a_new_atlas_knows_home_and_three_galaxies() {
    let universe = Universe::new(7);
    let home = universe.home().star.galaxy;
    let galaxies = Atlas::new(&universe).known_galaxies(&universe);
    assert_eq!(galaxies.len(), 1 + HOME_NEIGHBOURS);
    assert_eq!(galaxies[0].id, home);
    let gap = |id: GalaxyId| reveal::galaxy_gap(&universe.galaxy(id).unwrap(), &galaxies[0]);
    let farthest = galaxies.iter().map(|g| gap(g.id)).fold(0.0, f64::max);
    let known: HashSet<GalaxyId> = galaxies.iter().map(|g| g.id).collect();
    let others = universe.galaxies_near(home, 3);
    let unknown = others.iter().filter(|other| !known.contains(&other.id));
    assert!(unknown.clone().all(|other| gap(other.id) >= farthest));
    assert!(unknown.count() > 0);
}

/// A new atlas knows the home system and every system within reach of it, and no other.
#[test]
fn a_new_atlas_knows_only_systems_near_home() {
    let universe = Universe::new(7);
    let home = universe.home().star;
    let atlas = Atlas::new(&universe);
    let origin = universe.star(home).unwrap().pos;
    let systems = atlas.known_systems(&universe, home.galaxy);
    let reach = REVEAL_GLY * GLY;
    assert!(systems.iter().any(|star| star.id == home));
    assert!(systems.iter().all(|s| s.pos.distance(origin) <= reach));
    let around = universe.stars_near(home.galaxy, origin, REVEAL_GLY);
    assert_eq!(systems.len(), around.len());
    let other = atlas.known_galaxies(&universe)[1].id;
    assert!(atlas.known_systems(&universe, other).is_empty());
}

/// Visiting a system adds only the systems within reach of it, and never removes one.
#[test]
fn visiting_a_system_adds_only_its_neighbours() {
    let universe = Universe::new(7);
    let home = universe.home().star;
    let mut atlas = Atlas::new(&universe);
    let before = atlas.known_systems(&universe, home.galaxy);
    let far = before.last().unwrap().clone();
    assert_ne!(far.id, home, "the home star has no neighbour to fly to");
    assert!(atlas.visit(Target::Star(far.id)));
    assert!(!atlas.visit(Target::Star(far.id)));
    let after = atlas.known_systems(&universe, home.galaxy);
    let old: HashSet<StarId> = before.iter().map(|star| star.id).collect();
    let new: HashSet<StarId> = after.iter().map(|star| star.id).collect();
    assert!(old.is_subset(&new));
    let added = after.iter().filter(|star| !old.contains(&star.id));
    assert!(
        added
            .clone()
            .all(|star| star.pos.distance(far.pos) <= REVEAL_GLY * GLY)
    );
    assert!(added.count() > 0);
    let around = universe.stars_near(home.galaxy, far.pos, REVEAL_GLY);
    assert!(around.iter().all(|star| new.contains(&star.id)));
}

/// Visiting a galaxy reveals the galaxies within two cells of it, and never removes one.
#[test]
fn visiting_a_galaxy_reveals_two_cells() {
    let universe = Universe::new(7);
    let mut atlas = Atlas::new(&universe);
    let before = atlas.known_galaxies(&universe);
    let visited = before[1].id;
    assert!(atlas.visit(Target::Galaxy(visited)));
    let after: HashSet<GalaxyId> = atlas
        .known_galaxies(&universe)
        .iter()
        .map(|g| g.id)
        .collect();
    assert!(before.iter().all(|galaxy| after.contains(&galaxy.id)));
    let around = universe.galaxies_near(visited, REVEAL_CELLS);
    assert!(around.len() > 1 && around.iter().all(|galaxy| after.contains(&galaxy.id)));
    let old: HashSet<GalaxyId> = before.iter().map(|galaxy| galaxy.id).collect();
    let added = after.iter().filter(|id| !old.contains(*id));
    assert!(
        added
            .clone()
            .all(|id| (id.0 - visited.0).abs().max_element() <= REVEAL_CELLS)
    );
    assert!(added.count() > 0);
}

/// An atlas with visits and a destination survives a trip through toml unchanged and stays small.
#[test]
fn the_atlas_round_trips_through_toml() {
    let universe = Universe::new(7);
    let home = universe.home();
    let mut atlas = Atlas::new(&universe);
    let near = atlas.known_systems(&universe, home.star.galaxy);
    for star in near.iter().take(40) {
        atlas.visit(Target::Star(star.id));
    }
    atlas.visit(Target::Galaxy(GalaxyId(bevy::math::IVec3::new(-3, 2, 1))));
    for destination in [
        Target::Body(home),
        Target::Star(home.star),
        Target::Galaxy(home.star.galaxy),
    ] {
        atlas.destination = Some(destination);
        let text = toml::to_string(&atlas).unwrap();
        assert_eq!(toml::from_str::<Atlas>(&text).unwrap(), atlas);
        assert!(
            text.len() < 100 * atlas.systems.len() + 600,
            "{} bytes",
            text.len()
        );
    }
    assert_eq!(toml::from_str::<Atlas>("").unwrap(), Atlas::default());
}
