//! Look tests inside the app: no node is rounded, and menus fall back to a plain dark veil
//! where the blurred backdrop is not available.

use bevy::prelude::*;

use super::app;
use super::flow::open;
use crate::menus::Menu;
use crate::menus::main_menu::Board;
use crate::ui::{Backdrop, Blurred, theme};

/// The menus that can be opened.
const MENUS: [Menu; 5] = [
    Menu::Main,
    Menu::Worlds,
    Menu::NewGame,
    Menu::Settings,
    Menu::Pause,
];

/// No node of any menu has a border radius.
#[test]
fn no_node_is_rounded() {
    let mut app = app("square");
    let mut seen = 0;
    for menu in MENUS {
        open(&mut app, menu);
        let mut nodes = app.world_mut().query::<&Node>();
        for node in nodes.iter(app.world()) {
            assert_eq!(node.border_radius, BorderRadius::ZERO, "{menu:?}");
            seen += 1;
        }
    }
    assert!(seen > 200, "only {seen} nodes were checked");
}

/// Without a renderer there is no blur material, and every menu keeps its dark veil, except
/// the main menu: it has no veil, so the title picture shows at full light, and its root is
/// clear.
#[test]
fn menus_fall_back_to_a_plain_veil() {
    let mut app = app("veil");
    assert!(app.world().resource::<Backdrop>().0.is_none());
    for menu in MENUS {
        open(&mut app, menu);
        let mut roots = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<Blurred>>();
        let veils: Vec<Color> = roots.iter(app.world()).map(|veil| veil.0).collect();
        match menu {
            Menu::Main => assert!(veils.is_empty(), "{veils:?}"),
            _ => assert_eq!(veils, [theme::SCRIM], "{menu:?}"),
        }
    }
}

/// Returns the entities above `entity` in the tree, nearest first.
fn above(world: &World, entity: Entity) -> Vec<Entity> {
    let mut list = Vec::new();
    let mut at = entity;
    while let Some(parent) = world.get::<ChildOf>(at) {
        at = parent.parent();
        list.push(at);
    }
    list
}

/// Every text of the main menu sits on its one panel, filled with `PANEL`: the token whose
/// contrast over a white picture the theme tests hold. No text of it has a shadow.
#[test]
fn the_main_menu_sits_on_a_panel() {
    let mut app = app("panel");
    open(&mut app, Menu::Main);
    let mut boards = app
        .world_mut()
        .query_filtered::<(Entity, &BackgroundColor), With<Board>>();
    let (board, fill) = boards.single(app.world()).unwrap();
    assert_eq!(fill.0, theme::PANEL);
    let mut texts = app.world_mut().query_filtered::<Entity, With<Text>>();
    let texts: Vec<Entity> = texts.iter(app.world()).collect();
    let world = app.world();
    let mut seen = 0;
    for text in texts {
        let over = above(world, text);
        let in_menu = over
            .iter()
            .any(|e| world.get::<DespawnOnExit<Menu>>(*e).is_some());
        if !in_menu {
            continue;
        }
        assert!(
            over.contains(&board),
            "a text of the main menu is off its panel"
        );
        assert!(world.get::<TextShadow>(text).is_none());
        seen += 1;
    }
    assert!(seen >= 6, "only {seen} texts on the panel");
}
