//! The owner's rule of 2026-11-21: the in-game HUD has no panels. No node of it that holds a
//! text or an icon has a fill, the elements that had a panel pad nothing, and each corner
//! element starts on the edge margin. The minimap picture keeps its veil and its image.

use bevy::prelude::*;

use super::read::{square, under};
use super::rig::app;
use crate::menus::hud::HudRoot;
use crate::menus::hud::banner::BannerBlock;
use crate::menus::hud::climate::ClimateGauge;
use crate::menus::hud::compass::Bar;
use crate::menus::hud::coords::Coords;
use crate::menus::hud::minimap::widget::{Caption, Part};
use crate::menus::hud::panel::Instruments;
use crate::menus::hud::prompt::Prompt;
use crate::ui::theme;

/// The elements that sat on a panel or a strip before the rule.
type Bared = Or<(
    With<Coords>,
    With<ClimateGauge>,
    With<Instruments>,
    With<Prompt>,
    With<BannerBlock>,
    With<Caption>,
    With<Part>,
)>;

/// Returns an app with the HUD up, after four frames.
fn hud() -> App {
    let mut app = app("bare");
    for _ in 0..4 {
        app.update();
    }
    app
}

/// Returns whether `entity` is a text or holds one.
fn holds_text(world: &World, entity: Entity) -> bool {
    under(world, entity)
        .iter()
        .any(|below| world.get::<Text>(*below).is_some())
}

/// Returns the node of the parent of the one entity that carries `T`.
fn parent<T: Component>(app: &mut App) -> Node {
    let mut found = app.world_mut().query_filtered::<&ChildOf, With<T>>();
    let above = found.single(app.world()).expect("one of them").parent();
    app.world().get::<Node>(above).unwrap().clone()
}

/// No node of the HUD off the minimap picture that is or holds a text or an icon has a fill.
#[test]
fn no_hud_text_sits_on_a_fill() {
    let mut app = hud();
    let mut roots = app.world_mut().query_filtered::<Entity, With<HudRoot>>();
    let root = roots.single(app.world()).expect("one HUD");
    let picture = square(&mut app);
    let world = app.world();
    let drawn = under(world, picture);
    let mut seen = 0;
    for entity in under(world, root) {
        if drawn.contains(&entity) || !holds_text(world, entity) {
            continue;
        }
        let fill = world.get::<BackgroundColor>(entity).map(|fill| fill.0);
        let alpha = fill.map_or(0.0, |fill| fill.alpha());
        assert_eq!(alpha, 0.0, "{entity:?} sits on {fill:?}");
        seen += 1;
    }
    assert!(seen > 30, "only {seen} nodes hold a text");
}

/// The elements that had a panel or a strip pad nothing, so their text starts where they do.
#[test]
fn no_hud_element_pads_its_text() {
    let mut app = hud();
    let mut bared = app.world_mut().query_filtered::<&Node, Bared>();
    let nodes: Vec<&Node> = bared.iter(app.world()).collect();
    assert!(nodes.len() > 10, "only {} elements", nodes.len());
    for node in nodes {
        assert_eq!(node.padding, UiRect::ZERO, "{node:?}");
    }
}

/// Each corner element starts on the edge margin: the place lines top left, the compass at the
/// top, the minimap top right and the instruments with the frame rows bottom left.
#[test]
fn hud_elements_sit_on_the_edge_margin() {
    let mut app = hud();
    let edge = px(theme::EDGE);
    let place = parent::<Coords>(&mut app);
    assert_eq!((place.left, place.top), (edge, edge));
    let stack = parent::<Instruments>(&mut app);
    assert_eq!((stack.left, stack.bottom), (edge, edge));
    assert_eq!(stack.padding, UiRect::ZERO);
    assert_eq!(parent::<Bar>(&mut app).top, edge);
    let mut frames = app.world_mut().query::<(&Part, &Node)>();
    let (_, frame) = frames
        .iter(app.world())
        .find(|(part, _)| **part == Part::Frame)
        .expect("the minimap");
    assert_eq!((frame.top, frame.right), (edge, edge));
}
