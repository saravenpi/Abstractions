//! Computer tests: the zoom that frames a place, a double click that goes into it and sets
//! no course, and Escape letting go of the selection before it closes the map.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::pointer::framing;
use super::tests_pointer::rig;
use super::tests_rig::{game, run, tap};
use super::view::Slot;
use super::*;
use crate::universe::{GalaxyId, StarId};

/// A star frames its whole system on the system chart; a planet, a moon, a star on the galaxy
/// chart and a galaxy are framed at the dive zoom, from which the next notch goes inside.
#[test]
fn a_place_is_framed_at_its_own_zoom() {
    let rig = rig();
    let board = rig.world().resource::<Board>();
    let planet = board.lines[2].target;
    assert!(matches!(planet, Target::Body(_)));
    let star = Target::Star(StarId::default());
    assert_eq!(framing(Tab::System, star), 1.0);
    assert_eq!(framing(Tab::System, planet), DIVE);
    assert_eq!(framing(Tab::Galaxy, star), DIVE);
    assert_eq!(
        framing(Tab::Universe, Target::Galaxy(GalaxyId::default())),
        DIVE
    );
    let view = rig.world().resource::<MapView>();
    let lens = aimed(board, view, Dot::Row(2)).unwrap();
    let centre = view
        .dots
        .iter()
        .find(|(dot, _)| *dot == Dot::Row(2))
        .unwrap()
        .1;
    assert_eq!(lens, Lens { zoom: DIVE, centre }.clamped());
    assert_eq!(aimed(board, view, Dot::Row(0)).unwrap().zoom, 1.0);
    assert!(aimed(board, view, Dot::Row(board.lines.len())).is_none());
}

/// Two quick clicks on a row of the list ease the map onto its mark, ask to go into its
/// place, and set no course.
#[test]
fn a_double_click_on_a_row_goes_into_it_and_sets_no_course() {
    let mut app = rig();
    let mut slots = app.world_mut().query::<(Entity, &Slot)>();
    let entity = slots
        .iter(app.world())
        .find(|(_, slot)| slot.0 == 2)
        .unwrap()
        .0;
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.press(MouseButton::Left);
    app.world_mut().trigger(Activate { entity });
    app.world_mut().trigger(Activate { entity });
    let view = app.world().resource::<MapView>();
    let board = app.world().resource::<Board>();
    assert_eq!(Some(view.goal), aimed(board, view, Dot::Row(2)));
    assert!(view.picked && board.cursor == 2);
    assert_eq!(view.into, Some(board.lines[2].target));
    assert_eq!(app.world().resource::<Destination>().0, None);
}

/// With a mark clicked, the first Escape lets go of the row and the second closes the map.
#[test]
fn escape_lets_go_of_the_selection_before_it_closes() {
    let mut app = game("escape-selection");
    let open = |app: &App| *app.world().resource::<State<Computer>>().get() == Computer::Open;
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    let mut view = app.world_mut().resource_mut::<MapView>();
    (view.picked, view.marked) = (true, true);
    tap(&mut app, KeyCode::Escape, None);
    run(&mut app, 2);
    assert!(open(&app) && !app.world().resource::<MapView>().picked);
    tap(&mut app, KeyCode::Escape, None);
    run(&mut app, 2);
    assert!(!open(&app));
}
