//! Computer tests: the way home from a system, from a galaxy and from another galaxy, and the
//! Fly home on the card of the home planet, which flies the ship home in one command, in the
//! open panel.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3, IVec3};
use bevy::prelude::*;

use super::tests_rig::{game, run, tap};
use super::*;
use crate::space::{Anchor, Frame};
use crate::universe::{GALAXY_CELL, GalaxyId, LY, Universe};

/// Metres a position can be off by at the far end of a sector, where a 64-bit float steps by 16 m: the
/// star of the home is far from the corner of its sector, so its planet is placed to that.
const SECTOR_STEP: f64 = 16.0;

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
        (Frame::Galaxy(home.star.galaxy), star.pos + DVec3::X * LY),
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
    assert!((gap - planet.distance(system.1)).abs() < SECTOR_STEP);
    let (way, gap) = homeward(&universe, galaxy.0, galaxy.1, 0.0).unwrap();
    assert!(way.distance(DVec3::NEG_X) < 1e-6 && (gap / LY - 1.0).abs() < 1e-6);
    let (way, gap) = homeward(&universe, lost.0, lost.1, 0.0).unwrap();
    assert!(way.x < -0.9 && gap > 5.0 * GALAXY_CELL, "{way} {gap}");
    assert_eq!(
        homeward(&universe, Frame::Surface(home), DVec3::X, 0.0),
        None
    );
}

/// Counts the courses handed to the autopilot, and the asks to fly home to the base.
#[derive(Resource, Default)]
struct Engaged(usize, usize);

/// Puts the ship of `app` at the point `at` of `frame`, with no course and the map open, presses
/// Fly home on the card of the home planet, the Go button there, and returns the course and
/// whether the map is still open.
fn press_home(app: &mut App, (frame, at): (Frame, DVec3)) -> (Option<Target>, bool) {
    app.insert_resource(anchor(frame, at));
    app.world_mut().resource_mut::<Destination>().0 = None;
    tap(app, KeyCode::Tab, None);
    run(app, 3);
    let home = Target::Body(Universe::new(7).home());
    app.world_mut()
        .run_system_once(move |mut go: Go| go.press(Some(home)))
        .expect("the Go flow runs");
    run(app, 5);
    let open = *app.world().resource::<State<Computer>>().get() == Computer::Open;
    (app.world().resource::<Destination>().0, open)
}

/// In the open panel, Fly home on the card of the home planet sets the course to the home planet
/// and hands it to the autopilot in one command, from the home system, from elsewhere in the
/// home galaxy and from another galaxy, asks the ship to fly home to the base, and the map
/// closes each time. No key of its own does it. The saved atlas holds nothing but the visits and
/// the course.
#[test]
fn fly_home_on_the_map_flies_home_in_one_command() {
    let mut app = game("home");
    app.init_resource::<Engaged>();
    app.add_observer(|_: On<Engage>, mut engaged: ResMut<Engaged>| engaged.0 += 1);
    app.add_observer(|_: On<Homeward>, mut asked: ResMut<Engaged>| asked.1 += 1);
    let universe = Universe::new(7);
    let body = Target::Body(universe.home());
    for (count, place) in places(&universe).into_iter().enumerate() {
        let (course, open) = press_home(&mut app, place);
        assert_eq!((course, open), (Some(body), false));
        let engaged = app.world().resource::<Engaged>();
        assert_eq!((engaged.0, engaged.1), (count + 1, count + 1));
    }
    let saved = toml::to_string(app.world().resource::<Atlas>()).unwrap();
    let keys = saved.lines().filter_map(|line| line.split_once(" = "));
    let keys: Vec<&str> = keys.map(|(key, _)| key).collect();
    assert!(keys.iter().all(|key| !key.contains("home")), "{saved}");
}
