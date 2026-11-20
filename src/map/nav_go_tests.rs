//! Map tests: a double click that goes one level in, Back and Backspace that come out to
//! what was shown before, and the path that jumps to a level.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::Place;
use super::Trail;
use super::bar::{BackButton, Crumb, back_key, on_press};
use super::go::descend;
use super::tests::places;
use crate::computer::{Board, Lens, MapView, Tab, Target, Typing};
use crate::map::level::Level;
use crate::map::surface::Surface;

/// Builds an app that holds the levels of the map without drawing them, on the universe chart.
fn rig() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Level>()
        .init_resource::<MapView>()
        .init_resource::<Surface>()
        .init_resource::<Trail>()
        .init_resource::<Typing>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_observer(on_press);
    let mut board = Board::default();
    board.open(Tab::Universe, None);
    app.insert_resource(board);
    app
}

/// Asks the map to go into `target` as a double click does, once the chart eased onto it.
fn double_click(app: &mut App, target: Target) {
    let mut view = app.world_mut().resource_mut::<MapView>();
    view.into = Some(target);
    view.lens = view.goal;
    app.world_mut().run_system_once(descend).unwrap();
}

/// Returns the level shown: the tab of the chart and what it charts, and the level.
fn shown(app: &App) -> (Tab, Option<Target>, Level) {
    let board = app.world().resource::<Board>();
    (board.tab, board.inside, *app.world().resource::<Level>())
}

/// A double click goes exactly one level in: the universe into a galaxy, the galaxy into a
/// system, the system into a planet. A double click on the level shown stays.
#[test]
fn a_double_click_goes_one_level_in() {
    let (galaxy, star, body) = places();
    let mut app = rig();
    double_click(&mut app, Target::Galaxy(galaxy));
    assert_eq!(
        shown(&app),
        (Tab::Galaxy, Some(Target::Galaxy(galaxy)), Level::Chart)
    );
    double_click(&mut app, Target::Star(star));
    assert_eq!(
        shown(&app),
        (Tab::System, Some(Target::Star(star)), Level::Chart)
    );
    double_click(&mut app, Target::Star(star));
    assert_eq!(app.world().resource::<Trail>().stops.len(), 2);
    double_click(&mut app, Target::Body(body));
    assert_eq!(shown(&app).2, Level::Planet(body));
    let places: Vec<Place> = app
        .world()
        .resource::<Trail>()
        .stops
        .iter()
        .map(|stop| stop.place)
        .collect();
    assert_eq!(
        places,
        [Place::Universe, Place::Galaxy(galaxy), Place::System(star)]
    );
}

/// Presses Backspace for one frame of the key that goes back.
fn backspace(app: &mut App) {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::Backspace);
    app.world_mut().run_system_once(back_key).unwrap();
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::Backspace);
    keys.clear();
}

/// How the galaxy chart is looked at before going into a system.
const LOOKED: Lens = Lens {
    zoom: 3.0,
    centre: Vec2::new(0.2, -0.1),
};

/// Goes into the galaxy, looks at it through [`LOOKED`], then goes into the system.
fn into_the_system(app: &mut App) {
    let (galaxy, star, _) = places();
    double_click(app, Target::Galaxy(galaxy));
    app.world_mut().resource_mut::<MapView>().goal = LOOKED;
    double_click(app, Target::Star(star));
}

/// Backspace types while a search is typed, and goes back once nothing is.
#[test]
fn backspace_goes_back_unless_something_is_typed() {
    let mut app = rig();
    into_the_system(&mut app);
    *app.world_mut().resource_mut::<Typing>() = Typing::Search;
    backspace(&mut app);
    assert_eq!(shown(&app).0, Tab::System);
    *app.world_mut().resource_mut::<Typing>() = Typing::Off;
    backspace(&mut app);
    assert_eq!(shown(&app).0, Tab::Galaxy);
}

/// Backspace and the Back button go back to the level shown before and look at it as it was;
/// a part of the path jumps to its level, and Back then returns.
#[test]
fn back_returns_to_what_was_shown_before() {
    let (galaxy, _, _) = places();
    let mut app = rig();
    into_the_system(&mut app);
    backspace(&mut app);
    assert_eq!(
        shown(&app),
        (Tab::Galaxy, Some(Target::Galaxy(galaxy)), Level::Chart)
    );
    assert_eq!(app.world().resource::<MapView>().goal, LOOKED);
    let crumb = app.world_mut().spawn(Crumb(Place::Universe)).id();
    app.world_mut().trigger(Activate { entity: crumb });
    assert_eq!(shown(&app).0, Tab::Universe);
    let back = app.world_mut().spawn(BackButton).id();
    app.world_mut().trigger(Activate { entity: back });
    assert_eq!(
        shown(&app),
        (Tab::Galaxy, Some(Target::Galaxy(galaxy)), Level::Chart)
    );
    assert_eq!(app.world().resource::<MapView>().goal, LOOKED);
    backspace(&mut app);
    backspace(&mut app);
    assert_eq!(shown(&app).0, Tab::Universe);
}
