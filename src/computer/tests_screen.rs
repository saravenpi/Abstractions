//! Computer tests: the card written on the screen, in an app without a window.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::readout::{Facts, Subtitle, Title, shown, write};
use super::tests_detail::{records, world};
use super::*;
use crate::universe::Universe;

/// The name, the subtitle and the rows of a card on the screen.
type Parts = [Entity; 3];

/// Returns an app with an empty table and the universe of seed 7, and the parts of its card.
fn rig() -> (App, Parts) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Board>()
        .insert_resource(Universe::new(7));
    let title = app.world_mut().spawn((Title, Text::default())).id();
    let subtitle = app.world_mut().spawn((Subtitle, Text::default())).id();
    let facts = app.world_mut().spawn((Facts, Node::default())).id();
    (app, [title, subtitle, facts])
}

/// Writes the card and returns its name, its subtitle, how many rows it shows and how many
/// rows the card of the cursor row has.
fn read(app: &mut App, [title, subtitle, facts]: Parts) -> (String, String, usize, usize) {
    app.world_mut().run_system_once(write).unwrap();
    let said = |entity: Entity| app.world().get::<Text>(entity).unwrap().0.clone();
    let rows = app
        .world()
        .get::<Children>(facts)
        .map_or(0, |rows| rows.len());
    let universe = *app.world().resource::<Universe>();
    let odds = &mut Default::default();
    let wanted = shown(app.world().resource::<Board>(), &universe, odds)
        .rows
        .len();
    (said(title), said(subtitle), rows, wanted)
}

/// On the screen the card follows the cursor: the name, what the place is and one row per
/// fact of a star, then of a planet, and nothing for an empty table.
#[test]
fn the_screen_shows_the_card_of_the_cursor_row() {
    let (mut app, parts) = rig();
    assert_eq!(read(&mut app, parts), (String::new(), String::new(), 0, 0));
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
    let (name, kind, rows, wanted) = read(&mut app, parts);
    assert_eq!((name.as_str(), rows), ("Vereya", wanted));
    assert!(
        kind.starts_with("Class ") && kind.contains(" star in "),
        "{kind}"
    );
    app.world_mut().resource_mut::<Board>().step(1);
    let (name, kind, rows, wanted) = read(&mut app, parts);
    assert_eq!((name.as_str(), rows), ("Kodya", wanted));
    assert!(kind.starts_with("Ice planet of "), "{kind}");
}
