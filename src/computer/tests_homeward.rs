//! Computer tests: the way home from a system, from a galaxy and from another galaxy, and the
//! key that sets the course along it, in the open panel.

use bevy::math::{DQuat, DVec3, IVec3};
use bevy::prelude::*;

use super::tests_rig::{game, run, tap};
use super::view::Crumbs;
use super::*;
use crate::space::{Anchor, Frame};
use crate::universe::{GLY, GalaxyId, Universe};

/// Returns a galaxy that exists about ten cells east of the home galaxy.
fn far_galaxy(universe: &Universe) -> GalaxyId {
    let cell = universe.home().star.galaxy.0 + IVec3::new(10, 0, 0);
    universe.galaxies_near(GalaxyId(cell), 1)[0].id
}

/// Returns an anchor at the point `origin` of `frame`, with the render axes those of the frame.
fn anchor(frame: Frame, origin: DVec3) -> Anchor {
    Anchor {
        frame,
        origin,
        rotation: DQuat::IDENTITY,
        flat: false,
    }
}

/// Returns the three places a ship can be away from the home planet: in the home system, in
/// the home galaxy one light-year east of the home star, and in the middle of a far galaxy.
fn places(universe: &Universe) -> [(Frame, DVec3); 3] {
    let home = universe.home();
    let star = universe.star(home.star).unwrap();
    [
        (Frame::System(home.star), DVec3::X * 1.0e9),
        (Frame::Galaxy(home.star.galaxy), star.pos + DVec3::X * GLY),
        (Frame::Galaxy(far_galaxy(universe)), DVec3::ZERO),
    ]
}

/// The next place on the way home is the home planet inside the home system, the home star
/// elsewhere in the home galaxy and the home galaxy from another one, each on the tab that
/// lists it, and its direction and distance are those of its centre.
#[test]
fn the_way_home_picks_the_right_level() {
    let universe = Universe::new(7);
    let home = universe.home();
    let wanted = homes(&universe).map(Option::unwrap);
    let tabs = [Tab::System, Tab::Galaxy, Tab::Universe];
    for (index, (frame, at)) in places(&universe).into_iter().enumerate() {
        let target = toward(here(&universe, frame, at), &universe);
        assert_eq!((target, tab_of(target)), (wanted[index], tabs[index]));
        assert!(homeward(&universe, frame, at, 30.0).is_some());
    }
    let [system, galaxy, lost] = places(&universe);
    let planet = universe.planet_position(home, 30.0);
    let (way, gap) = homeward(&universe, system.0, system.1, 30.0).unwrap();
    assert!(way.distance((planet - system.1).normalize()) < 1e-9);
    assert!((gap - planet.distance(system.1)).abs() < 1.0);
    let (way, gap) = homeward(&universe, galaxy.0, galaxy.1, 0.0).unwrap();
    assert!(way.distance(DVec3::NEG_X) < 1e-6 && (gap / GLY - 1.0).abs() < 1e-6);
    let (way, gap) = homeward(&universe, lost.0, lost.1, 0.0).unwrap();
    assert!(way.x < -0.9 && gap > 8.0e15, "{way} {gap}");
    assert_eq!(
        homeward(&universe, Frame::Surface(home), DVec3::X, 0.0),
        None
    );
}

/// Puts the ship of `app` at the point `at` of `frame`, presses the Home key and returns the
/// course, the open tab, the target of the selected row and the header.
fn press_home(app: &mut App, (frame, at): (Frame, DVec3)) -> (Option<Target>, Tab, Target, String) {
    app.insert_resource(anchor(frame, at));
    run(app, 2);
    tap(app, KeyCode::Home, None);
    run(app, 40);
    let board = app.world().resource::<Board>();
    let row = (board.tab, board.lines[board.cursor].target);
    let course = app.world().resource::<Destination>().0;
    let mut crumbs = app.world_mut().query_filtered::<&Text, With<Crumbs>>();
    let header = crumbs.single(app.world()).unwrap().0.clone();
    (course, row.0, row.1, header)
}

/// In the open panel, the Home key sets the course to the home galaxy from another galaxy, to
/// the home star from elsewhere in the home galaxy and to the home planet in the home system,
/// and each time opens the tab that lists it with the cursor on its row. The header says how
/// far home is, and the saved atlas holds nothing but the visits and the course.
#[test]
fn the_home_key_sets_the_course_one_level_at_a_time() {
    let mut app = game("home");
    let universe = Universe::new(7);
    let [body, star, galaxy] = homes(&universe).map(Option::unwrap);
    let [system, near, lost] = places(&universe);
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 3);
    let (course, tab, row, header) = press_home(&mut app, lost);
    assert_eq!((course, tab, row), (Some(galaxy), Tab::Universe, galaxy));
    assert_eq!(app.world().resource::<Board>().cursor, 0);
    assert!(header.ends_with("galaxies from home"), "{header}");
    let (course, tab, row, header) = press_home(&mut app, near);
    assert_eq!((course, tab, row), (Some(star), Tab::Galaxy, star));
    assert_eq!(app.world().resource::<Board>().cursor, 0);
    assert!(header.ends_with("1.00 gly from home"), "{header}");
    let (course, tab, row, header) = press_home(&mut app, system);
    assert_eq!((course, tab, row), (Some(body), Tab::System, body));
    assert!(!header.contains("from home"), "{header}");
    let saved = toml::to_string(app.world().resource::<Atlas>()).unwrap();
    let keys = saved.lines().filter_map(|line| line.split_once(" = "));
    let keys: Vec<&str> = keys.map(|(key, _)| key).collect();
    assert!(keys.iter().all(|key| !key.contains("home")), "{saved}");
}
