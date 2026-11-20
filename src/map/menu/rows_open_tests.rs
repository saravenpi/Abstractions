//! Map tests: a right click on a row of the side panel opens the menu of what the row stands
//! for at the pointer, inside the window.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;

use super::super::rig::{board, friend};
use super::super::{ContextMenu, Opened, Under, WIDTH};
use super::summon;
use crate::computer::{Slot, Target};
use crate::map::favourite::Favourites;
use crate::map::side::{FavRow, PlayerRow};
use crate::menus::{MarkerPlace, Markers};
use crate::universe::{BodyId, StarId};

/// Returns an app with a 960 by 540 window whose pointer is at `at`, a right click pressed,
/// the board of [`board`], two favourites and one friend on a planet.
fn app(at: Vec2) -> App {
    let mut app = App::new();
    let mut window = Window {
        resolution: (960, 540).into(),
        ..default()
    };
    window.set_cursor_position(Some(at));
    app.world_mut().spawn((window, PrimaryWindow));
    let mut buttons = ButtonInput::<MouseButton>::default();
    buttons.press(MouseButton::Right);
    let mut favourites = Favourites::default();
    favourites.save(BodyId::default(), DVec3::X);
    favourites.save(BodyId::default(), DVec3::Y);
    let markers = Markers(vec![friend(MarkerPlace::Body(BodyId::default(), DVec3::Z))]);
    app.insert_resource(buttons)
        .insert_resource(board())
        .insert_resource(favourites)
        .insert_resource(markers)
        .init_resource::<ContextMenu>();
    app
}

/// Spawns a row the pointer is over or not, with `part`.
fn spawn(app: &mut App, part: impl Bundle, over: bool) {
    let cursor = RelativeCursorPosition {
        cursor_over: over,
        normalized: None,
    };
    app.world_mut().spawn((part, cursor));
}

/// Returns the menu a right click opens with the pointer at `at` over the row `part`, the other
/// rows of each list under it but not pointed at.
fn opened(at: Vec2, part: impl Bundle) -> Option<Opened> {
    let mut app = app(at);
    spawn(&mut app, Slot(2), false);
    spawn(&mut app, FavRow(0), false);
    spawn(&mut app, PlayerRow(0), false);
    spawn(&mut app, part, true);
    app.world_mut().run_system_once(summon).unwrap();
    app.world().resource::<ContextMenu>().0
}

/// A right click on a result, a favourite or a player opens its menu at the pointer, and one
/// near the bottom right corner opens it moved inside the window. A click over no row opens
/// nothing.
#[test]
fn a_right_click_on_a_row_opens_its_menu() {
    let at = Vec2::new(700.0, 100.0);
    let result = opened(at, Slot(1)).unwrap();
    assert_eq!(result.under, Under::Place(Target::Star(StarId::default())));
    assert_eq!(result.at, at);
    assert_eq!(opened(at, FavRow(1)).unwrap().under, Under::Favourite(1));
    let player = opened(at, PlayerRow(0)).unwrap();
    assert_eq!(player.under, Under::Player(0, BodyId::default()));
    let corner = opened(Vec2::new(959.0, 539.0), FavRow(0)).unwrap();
    assert!(corner.at.x + WIDTH <= 960.0 && corner.at.y < 539.0);
    assert_eq!(opened(at, Node::default()), None);
}
