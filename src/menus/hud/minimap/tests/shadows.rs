//! The owner's rule on shadows of 2026-11-21: the in-game HUD has no panels, so every text and
//! icon of it carries exactly the HUD shadow of its colour. Nothing drawn on the minimap picture
//! carries one, nor the map, nor a menu, and no entity anywhere carries a `BoxShadow`.

use bevy::prelude::*;

use super::read::{square, under};
use super::rig::{app, tap};
use crate::computer::Computer;
use crate::menus::Menu;
use crate::menus::hud::HudRoot;
use crate::space::{Location, Travel};
use crate::ui::{OverWorld, hud_shadow};

/// Runs `frames` frames of `app`.
fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

/// Returns every text with a shadow that lies under no root over the world, and every entity
/// with a box shadow.
fn stray(app: &mut App) -> Vec<Entity> {
    let mut roots = app.world_mut().query_filtered::<Entity, With<OverWorld>>();
    let mut texts = app.world_mut().query_filtered::<Entity, With<TextShadow>>();
    let mut boxes = app.world_mut().query_filtered::<Entity, With<BoxShadow>>();
    let world = app.world();
    let over: Vec<Entity> = roots
        .iter(world)
        .flat_map(|root| under(world, root))
        .collect();
    let mut found: Vec<Entity> = texts
        .iter(world)
        .filter(|text| !over.contains(text))
        .collect();
    found.extend(boxes.iter(world));
    found
}

/// Every text and icon of the HUD off the minimap picture carries the HUD shadow of its colour
/// at the scale of the headless app, one pixel down and to the right; nothing on the picture
/// carries a shadow: not its marks, not the north letter, not the arrow of the player.
#[test]
fn hud_text_carries_the_hud_shadow_and_the_minimap_none() {
    let mut app = app("shadows");
    run(&mut app, 4);
    let mut roots = app.world_mut().query_filtered::<Entity, With<HudRoot>>();
    let root = roots.single(app.world()).expect("one HUD");
    let picture = square(&mut app);
    let drawn = under(app.world(), picture);
    let world = app.world();
    let mut shaded = 0;
    for entity in under(world, root) {
        let Some(text) = world.get::<Text>(entity) else {
            continue;
        };
        let shadow = world.get::<TextShadow>(entity).copied();
        if drawn.contains(&entity) {
            assert_eq!(shadow, None, "{text:?} on the minimap casts a shadow");
            continue;
        }
        let color = world.get::<TextColor>(entity).unwrap().0;
        assert_eq!(shadow, Some(hud_shadow(color, 1.0)), "{text:?}");
        shaded += 1;
    }
    assert!(shaded > 20, "only {shaded} texts of the HUD");
    assert!(
        drawn
            .iter()
            .any(|entity| world.get::<Text>(*entity).is_some())
    );
}

/// With the HUD up, with the computer open on the map, and with each menu open, no shadow lies
/// outside the HUD: menus and the map carry none, and nothing carries a box shadow.
#[test]
fn nothing_but_the_hud_casts_a_shadow() {
    let mut app = app("strays");
    run(&mut app, 4);
    assert!(stray(&mut app).is_empty(), "the HUD casts a stray shadow");
    app.world_mut().resource_mut::<Location>().travel = Travel::InShip;
    tap(&mut app, KeyCode::Tab);
    assert_eq!(
        *app.world().resource::<State<Computer>>().get(),
        Computer::Open
    );
    run(&mut app, 4);
    assert!(stray(&mut app).is_empty(), "the map casts a shadow");
    let mut next = app.world_mut().resource_mut::<NextState<Computer>>();
    next.set(Computer::Closed);
    for menu in [
        Menu::Pause,
        Menu::Main,
        Menu::Worlds,
        Menu::NewGame,
        Menu::Settings,
    ] {
        app.world_mut().resource_mut::<NextState<Menu>>().set(menu);
        run(&mut app, 4);
        assert!(stray(&mut app).is_empty(), "{menu:?} casts a shadow");
    }
}
