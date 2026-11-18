//! Computer tests: the readout written on the screen, in an app without a window.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::detail::LINES;
use super::readout::{Readout, shown, write};
use super::tests_detail::{records, world};
use super::*;

/// On the screen the readout follows the cursor: the record of a planet, the facts of a
/// star, and nothing for an empty table.
#[test]
fn the_screen_shows_the_readout_of_the_cursor_row() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).init_resource::<Board>();
    let text = app.world_mut().spawn((Readout, Text::default())).id();
    let read = |app: &mut App| {
        app.world_mut().run_system_once(write).unwrap();
        app.world().get::<Text>(text).unwrap().0.clone()
    };
    assert_eq!(read(&mut app), "");
    let (planet_row, planet) = world(records()[3]);
    let star = Line {
        title: "Vereya".to_string(),
        detail: "2.00 Mm - class G - 5 planets - visited".to_string(),
        target: Target::Star(planet.id.star),
        ..planet_row.clone()
    };
    let mut board = app.world_mut().resource_mut::<Board>();
    board.worlds = vec![None, Some(planet)];
    board.fill("Vereya system".to_string(), vec![star, planet_row]);
    assert_eq!(read(&mut app), "Vereya\nclass G - 5 planets");
    app.world_mut().resource_mut::<Board>().step(1);
    let board = app.world().resource::<Board>().clone();
    let shown_now = read(&mut app);
    assert_eq!(shown_now, shown(&board));
    assert!(shown_now.starts_with("Kodya\nice - -120 C - tilt 57 deg\nair thin"));
    assert_eq!(shown_now.lines().count(), LINES);
}
