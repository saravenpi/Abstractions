//! Computer tests: home is always known and marked, and its row is pinned first away from it.

use bevy::math::{DVec3, IVec3};

use super::route::HOME_LABEL;
use super::sift::{arrange, matches};
use super::*;
use crate::space::UniPos;
use crate::universe::{GALAXY_CELL, GalaxyId, Universe};

/// Returns the galaxy cell ten cells east of the home galaxy and the middle of that cell.
pub(super) fn far(universe: &Universe) -> (GalaxyId, UniPos) {
    let cell = universe.home().star.galaxy.0 + IVec3::new(10, 0, 0);
    (
        GalaxyId(cell),
        UniPos::new(cell, DVec3::splat(0.5 * GALAXY_CELL)),
    )
}

/// Returns an atlas that only ever saw the far cell: nothing of home was visited.
pub(super) fn stranger(universe: &Universe) -> Atlas {
    let mut atlas = Atlas::default();
    atlas.visit(Target::Galaxy(far(universe).0));
    atlas
}

/// Ten galaxy cells from home, with an atlas that never visited it, the home galaxy, the home
/// star and the home planet are still listed, each with the word home after its name.
#[test]
fn home_is_known_ten_galaxies_away() {
    let universe = Universe::new(7);
    let (home, atlas) = (universe.home(), stranger(&universe));
    let view = View {
        universe: &universe,
        atlas: &atlas,
        ship: far(&universe).1,
        time: 0.0,
    };
    let named = |lines: &[Line], target: Target| {
        let row = lines.iter().find(|line| line.target == target);
        row.is_some_and(|line| line.title.ends_with(HOME_LABEL))
    };
    assert!(named(&view.universe(), Target::Galaxy(home.star.galaxy)));
    let stars = view.galaxy(home.star.galaxy, std::iter::empty());
    assert!(stars.len() == 1 && named(&stars, Target::Star(home.star)));
    assert!(named(&view.system(home.star), Target::Body(home)));
    let known = atlas.known_systems(&universe, home.star.galaxy);
    assert!(known.iter().any(|star| star.id == home.star));
    assert_eq!(homes(&universe)[0], Some(Target::Body(home)));
}

/// Away from the home galaxy its row is the first of the universe tab whatever the order and
/// the filter, even a filter no name holds, and typing "home" matches it.
#[test]
fn the_home_row_is_pinned_through_sort_and_filter() {
    let universe = Universe::new(7);
    let (home, atlas) = (universe.home(), stranger(&universe));
    let (cell, ship) = far(&universe);
    let (universe, atlas) = (&universe, &atlas);
    let lines = View {
        universe,
        atlas,
        ship,
        time: 0.0,
    }
    .universe();
    let ours = Target::Galaxy(home.star.galaxy);
    let here = [None, None, Some(Target::Galaxy(cell))];
    let pin = pinned(Tab::Universe, here, universe);
    assert_eq!(pin, Some(ours));
    let plain = arrange(lines.clone(), "", Sort::Near);
    assert!(plain.len() > 3 && plain[0].target != ours);
    for (filter, sort) in [("", Sort::Near), ("", Sort::Name), ("home", Sort::Name)] {
        let rows = arranged(lines.clone(), pin, filter, sort);
        let count = rows.iter().filter(|line| line.target == ours).count();
        assert_eq!((rows[0].target, count), (ours, 1), "{filter} {sort:?}");
        assert!(matches(&rows[0].title, "home"));
    }
    assert_eq!(arranged(lines.clone(), pin, "zzzz", Sort::Near).len(), 1);
    assert_eq!(arranged(lines.clone(), None, "", Sort::Near), plain);
}

/// At home nothing is pinned, in the home galaxy away from the home system the home star
/// is, and the system tab pins nothing.
#[test]
fn home_is_pinned_only_away_from_it() {
    let universe = Universe::new(7);
    let at_home = homes(&universe);
    let wandering = [None, None, at_home[2]];
    assert_eq!(pinned(Tab::Universe, at_home, &universe), None);
    assert_eq!(pinned(Tab::Universe, [None; 3], &universe), at_home[2]);
    assert_eq!(pinned(Tab::Galaxy, wandering, &universe), at_home[1]);
    assert_eq!(pinned(Tab::Galaxy, at_home, &universe), None);
    assert_eq!(pinned(Tab::System, [None; 3], &universe), None);
}
