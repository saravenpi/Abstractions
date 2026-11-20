//! The minimap inside the app: it is part of the HUD and hides with it, its marks sit where
//! their targets lie, and its image turns and slides with the player.

use std::f32::consts::FRAC_PI_2;

use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::ui::Val2;

use super::read::{at, caption, furnish, part, turn};
use super::rig::{app, ground, look, settle, tap};
use crate::computer::Computer;
use crate::map::raster::colour;
use crate::menus::Menu;
use crate::menus::hud::HudRoot;
use crate::menus::hud::minimap::patch::TEXELS;
use crate::menus::hud::minimap::scope::Scope;
use crate::menus::hud::minimap::widget::{Part, SIDE};
use crate::space::{Location, Travel};
use crate::ui::{Blurred, theme};
use crate::world::Planet;

/// Returns whether the HUD is shown, after three more frames.
fn hud_shown(app: &mut App) -> bool {
    for _ in 0..3 {
        app.update();
    }
    let mut roots = app
        .world_mut()
        .query_filtered::<&Visibility, With<HudRoot>>();
    *roots.single(app.world()).expect("one HUD") == Visibility::Inherited
}

/// The minimap is a square without a border of the HUD in the top right corner, as far from the edges
/// as the compass is from the top, over a dark veil where the game cannot be blurred.
#[test]
fn the_minimap_is_a_square_of_the_hud() {
    let mut app = app("square");
    let frame = part(&mut app, Part::Frame);
    let (margin, side) = (px(theme::EDGE), Val::Vh(100.0 * SIDE / 1080.0));
    assert_eq!((frame.top, frame.right), (margin, margin));
    assert_eq!((frame.width, frame.height), (side, side));
    assert_eq!(frame.border, UiRect::ZERO);
    assert_eq!(frame.overflow, Overflow::clip());
    let mut owners = app.world_mut().query::<(&Part, &ChildOf)>();
    let mut frames = owners
        .iter(app.world())
        .filter(|(part, _)| **part == Part::Frame);
    let root = frames.next().expect("the frame has a parent").1.parent();
    assert!(app.world().get::<HudRoot>(root).is_some());
    let mut veils = app
        .world_mut()
        .query_filtered::<&BackgroundColor, With<Blurred>>();
    for _ in 0..200 {
        if veils.single(app.world()).unwrap().0 == theme::SCRIM {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
        app.update();
    }
    assert_eq!(veils.single(app.world()).unwrap().0, theme::SCRIM);
}

/// Opens `menu` over the game, or closes the menus with `Menu::None`.
fn open(app: &mut App, menu: Menu) {
    app.world_mut().resource_mut::<NextState<Menu>>().set(menu);
}

/// The minimap shows while playing, and hides with the HUD behind the pause menu and behind
/// the open computer, which has the large map.
#[test]
fn the_minimap_hides_behind_menus_and_the_computer() {
    let mut app = app("hidden");
    assert!(hud_shown(&mut app));
    open(&mut app, Menu::Pause);
    assert!(!hud_shown(&mut app));
    open(&mut app, Menu::None);
    assert!(hud_shown(&mut app));
    app.world_mut().resource_mut::<Location>().travel = Travel::InShip;
    tap(&mut app, KeyCode::Tab);
    let computer = *app.world().resource::<State<Computer>>().get();
    assert_eq!(computer, Computer::Open);
    assert!(!hud_shown(&mut app));
    let mut next = app.world_mut().resource_mut::<NextState<Computer>>();
    next.set(Computer::Closed);
    assert!(hud_shown(&mut app));
}

/// Looking north from the start, the arrow is in the middle, the home base 100 m ahead is
/// above it, the ship a kilometre east waits on the right rim and north is at the top.
#[test]
fn the_marks_sit_where_their_targets_lie() {
    let mut app = app("marks");
    furnish(&mut app);
    assert_eq!(settle(&mut app), 250.0);
    assert_eq!(at(&mut app, Part::Arrow), Some((50, 50)));
    assert_eq!(at(&mut app, Part::Home), Some((50, 30)));
    assert_eq!(at(&mut app, Part::Ship), Some((93, 50)));
    assert_eq!(at(&mut app, Part::North), Some((50, 7)));
    assert_eq!(caption(&mut app), "250 m");
}

/// The image of the ground is painted by the worker, twice as wide as the frame around its
/// middle, and neither moved nor turned while the player stands at its middle looking north. Each texel has the
/// colour of the full map there: a kilometre of a planet of real size is too even to tell apart by
/// its relief, so the test reads the colours where they are painted.
#[test]
fn the_ground_is_painted_around_the_player() {
    let mut app = app("ground");
    settle(&mut app);
    let image = part(&mut app, Part::Ground);
    let (side, corner) = (percent(200), percent(-50));
    assert_eq!(image.display, Display::Flex);
    assert_eq!((image.width, image.height), (side, side));
    assert_eq!((image.left, image.top), (corner, corner));
    let placed = turn(&mut app);
    assert!(placed.rotation.as_radians().abs() < 1e-3, "{placed:?}");
    let patch = app.world().resource::<Scope>().patch.expect("a patch");
    let globe = app.world().resource::<Planet>().globe.clone();
    let handle = app.world().resource::<Scope>().image.clone();
    let painted = app.world().resource::<Assets<Image>>().get(&handle);
    let texels = painted.unwrap().data.as_ref().unwrap().as_chunks::<4>().0;
    assert!(texels.iter().all(|texel| texel[3] == 255));
    for (x, y) in [(0, 0), (48, 48), (95, 20), (30, 95)] {
        let wanted = colour(&globe, patch.dir(x, y));
        assert_eq!(texels[y * TEXELS + x], wanted, "({x}, {y})");
    }
}

/// Turning east turns the map: the base is now to the left, the ship ahead and north on the
/// left rim. Walking north slides the image down without painting a new one.
#[test]
fn the_map_turns_and_slides_with_the_player() {
    let mut app = app("turns");
    furnish(&mut app);
    settle(&mut app);
    let painted = app.world().resource::<Scope>().patch;
    look(&mut app, ground(0.0, 0.0), DVec3::NEG_Z);
    assert_eq!(at(&mut app, Part::Home), Some((30, 50)));
    assert_eq!(at(&mut app, Part::Ship), Some((50, 7)));
    assert_eq!(at(&mut app, Part::North), Some((7, 50)));
    assert_eq!(at(&mut app, Part::Arrow), Some((50, 50)));
    assert!((turn(&mut app).rotation.as_radians() + FRAC_PI_2).abs() < 1e-3);
    look(&mut app, ground(50.0, 0.0), DVec3::Y);
    assert_eq!(at(&mut app, Part::Home), Some((50, 40)));
    let Val2 { x, y } = turn(&mut app).translation;
    assert!(
        matches!(x, Val::Percent(right) if right.abs() < 0.01),
        "{x:?}"
    );
    assert!(
        matches!(y, Val::Percent(down) if (down - 5.0).abs() < 0.01),
        "{y:?}"
    );
    assert_eq!(app.world().resource::<Scope>().patch, painted);
    assert!(app.world().resource::<Scope>().job.is_none());
}
