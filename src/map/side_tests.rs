//! Map tests: the pins and players in the side panel say what they hold in the exact words, hide
//! what is not there, and fit the room of the panel.

use std::sync::{Arc, Mutex};

use bevy::math::DVec3;
use bevy::prelude::*;

use super::places::Places;
use super::side::{ALONE, NoPlayers, PinActions, PinHint, PinRow, Players, pins, players};
use super::side_show::{PICK, words, write};
use crate::computer::{Board, GLYPH, ROOM, Typing};
use crate::menus::Markers;
use crate::net::{Link, Role, Script, Source, Wire};
use crate::ui::theme;
use crate::universe::Universe;

/// Returns a headless app that holds the side panel and writes it.
fn rig() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Places>()
        .init_resource::<Typing>()
        .init_resource::<Markers>()
        .init_resource::<Board>()
        .insert_resource(Universe::new(3))
        .add_systems(Update, write);
    app.world_mut().spawn(pins());
    app.world_mut().spawn(players());
    app.update();
    app
}

/// Returns the line under the pins.
fn hint(app: &mut App) -> String {
    let world = app.world_mut();
    let mut lines = world.query_filtered::<&Text, With<PinHint>>();
    lines.single(world).unwrap().0.clone()
}

/// Returns whether the node of the marker `M` is shown.
fn shown<M: Component>(app: &mut App) -> bool {
    let world = app.world_mut();
    let mut nodes = world.query_filtered::<&Node, With<M>>();
    nodes.single(world).unwrap().display != Display::None
}

/// With no pin the panel says how to drop one, with pins and none selected it says to click one,
/// and with one selected it says what it does and shows its three buttons.
#[test]
fn the_pins_say_what_to_do() {
    let mut app = rig();
    assert_eq!(
        hint(&mut app),
        "No pins yet. Press B or right-click the ground to drop one."
    );
    assert!(words(0, None, "N").contains("Press N or right-click"));
    assert!(!shown::<PinActions>(&mut app));
    let home = Universe::new(3).home();
    app.world_mut()
        .resource_mut::<Places>()
        .drop(home, DVec3::X);
    app.update();
    assert_eq!(hint(&mut app), PICK);
    app.world_mut().resource_mut::<Places>().selected = Some(0);
    app.update();
    assert!(hint(&mut app).starts_with("Flies to "));
    assert!(shown::<PinActions>(&mut app));
}

/// Only the rows of the pins that exist are shown, with their names.
#[test]
fn rows_past_the_last_pin_are_hidden() {
    let mut app = rig();
    let home = Universe::new(3).home();
    app.world_mut()
        .resource_mut::<Places>()
        .drop(home, DVec3::X);
    app.update();
    let world = app.world_mut();
    let mut rows = world.query::<(&PinRow, &Node)>();
    let shown: Vec<usize> = rows
        .iter(world)
        .filter(|(_, node)| node.display != Display::None)
        .map(|(row, _)| row.0)
        .collect();
    assert_eq!(shown, [0]);
}

/// The players are hidden outside a session, and in one they say no one else is there.
#[test]
fn the_players_say_when_alone() {
    let mut app = rig();
    assert!(!shown::<Players>(&mut app));
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    app.world_mut()
        .insert_resource(Link::over(Source::Script(script), Role::Host));
    app.update();
    assert!(shown::<Players>(&mut app) && shown::<NoPlayers>(&mut app));
    assert_eq!(ALONE, "No one else is here yet.");
}

/// The line under a selected pin fits one line of the panel for any distance and any name.
#[test]
fn the_line_under_a_pin_fits_the_panel() {
    let line = (ROOM / GLYPH) as usize;
    for metres in [3.0, 999.0, 12_500.0, 3.2e9, 9.4e15] {
        let said = words(1, Some(("Kolva", Some(metres))), "B");
        assert!(said.chars().count() <= line, "{said}");
    }
    let said = words(1, Some(("Wonderland of Seven Names", None)), "B");
    assert!(said.chars().count() <= line, "{said}");
}

/// The three buttons of a pin sit on one row of the panel.
#[test]
fn the_pin_buttons_fit_one_row() {
    let width = |words: &str| words.chars().count() as f32 * GLYPH + 2.0 * theme::GAP_S;
    let row = width("Go") + width("Rename [R]") + width("Remove [Del]") + 2.0 * theme::GAP_S;
    assert!(row <= ROOM, "{row}");
}
