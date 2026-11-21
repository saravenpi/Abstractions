//! Scroll tests: the focus is brought into view by the least move, the bar follows the offset,
//! and the wheel moves only the column under the pointer.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, RelativeCursorPosition};

use super::*;

/// A span above the view scrolls up, one below scrolls down, one inside stays.
#[test]
fn reveal_moves_only_as_far_as_needed() {
    assert_eq!(reveal(40.0, (90.0, 110.0), (100.0, 300.0)), 30.0);
    assert_eq!(reveal(40.0, (290.0, 320.0), (100.0, 300.0)), 60.0);
    assert_eq!(reveal(40.0, (150.0, 180.0), (100.0, 300.0)), 40.0);
}

/// The bar hides while the content fits, is as tall as the share in view, and moves from the
/// top to the bottom of the view with the offset, never past either end.
#[test]
fn the_bar_follows_the_offset() {
    assert_eq!(thumb(200.0, 200.0, 0.0), None);
    assert_eq!(thumb(200.0, 120.0, 0.0), None);
    assert_eq!(thumb(200.0, 800.0, 0.0), Some((0.0, 0.25)));
    assert_eq!(thumb(200.0, 800.0, 300.0), Some((0.375, 0.25)));
    assert_eq!(thumb(200.0, 800.0, 600.0), Some((0.75, 0.25)));
    assert_eq!(thumb(200.0, 800.0, 9000.0), Some((0.75, 0.25)));
}

/// Returns a column 100 pixels tall over 400 pixels of content, with the pointer over it or
/// not, or without a pointer at all.
fn column(app: &mut App, pointer: Option<bool>) -> Entity {
    let node = ComputedNode {
        size: Vec2::new(100.0, 100.0),
        content_size: Vec2::new(100.0, 400.0),
        inverse_scale_factor: 1.0,
        ..default()
    };
    let column = app
        .world_mut()
        .spawn((Scroller, ScrollPosition::default(), node))
        .id();
    if let Some(cursor_over) = pointer {
        app.world_mut()
            .entity_mut(column)
            .insert(RelativeCursorPosition {
                cursor_over,
                normalized: None,
            });
    }
    column
}

/// One notch of the wheel scrolls the column under the pointer and a column that does not track
/// the pointer, and leaves a column the pointer is not over where it is.
#[test]
fn the_wheel_scrolls_only_the_column_under_the_pointer() {
    let mut app = App::new();
    let (under, away, plain) = (
        column(&mut app, Some(true)),
        column(&mut app, Some(false)),
        column(&mut app, None),
    );
    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, -1.0),
    });
    app.world_mut().run_system_once(wheel).unwrap();
    let offset = |app: &App, column: Entity| app.world().get::<ScrollPosition>(column).unwrap().y;
    assert_eq!(offset(&app, under), theme::ROW_HEIGHT);
    assert_eq!(offset(&app, away), 0.0);
    assert_eq!(offset(&app, plain), theme::ROW_HEIGHT);
}
