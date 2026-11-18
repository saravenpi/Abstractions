//! Reading the minimap back out of the app: where each part is, what the corner says and what
//! is drawn on the sheet of the chart. Also the home base and the ship the marks point at.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::rig::ground;
use crate::home::Base;
use crate::menus::hud::minimap::widget::{Caption, Chart, Part};
use crate::ship::Ship;
use crate::space::Frame;
use crate::universe::Universe;

/// Puts the home base 100 m north of the start and the ship a kilometre east of it.
pub fn furnish(app: &mut App) {
    let body = app.world().resource::<Universe>().home();
    let centre = ground(100.0, 0.0);
    app.insert_resource(Base {
        body,
        centre,
        up: centre.normalize(),
        east: DVec3::NEG_Z,
        north: DVec3::Y,
        wake: centre,
        pad: centre,
        heading: DVec3::Y,
        door: centre,
    });
    let ship = Ship::at_rest(Frame::Surface(body), ground(0.0, 1000.0), DQuat::IDENTITY);
    app.world_mut().spawn(ship);
}

/// Returns the node of the only `wanted` part.
pub fn part(app: &mut App, wanted: Part) -> Node {
    let mut parts = app.world_mut().query::<(&Part, &Node)>();
    let found = parts.iter(app.world()).filter(|(part, _)| **part == wanted);
    let nodes: Vec<Node> = found.map(|(_, node)| node.clone()).collect();
    assert_eq!(nodes.len(), 1, "one {wanted:?}");
    nodes[0].clone()
}

/// Returns where the part `wanted` sits, as whole percentages from the left and from the top
/// of the map, or nothing while it is hidden.
pub fn at(app: &mut App, wanted: Part) -> Option<(i32, i32)> {
    let node = part(app, wanted);
    let share = |val: Val| match val {
        Val::Percent(share) => share.round() as i32,
        other => panic!("{wanted:?} is placed with {other:?}"),
    };
    (node.display != Display::None).then(|| (share(node.left), share(node.top)))
}

/// Returns the move and the turn of the image of the ground.
pub fn turn(app: &mut App) -> UiTransform {
    let mut parts = app.world_mut().query::<(&Part, &UiTransform)>();
    let mut grounds = parts
        .iter(app.world())
        .filter(|(part, _)| **part == Part::Ground);
    *grounds.next().expect("the image of the ground").1
}

/// Returns the text in the corner of the minimap.
pub fn caption(app: &mut App) -> String {
    let mut captions = app.world_mut().query_filtered::<&Text, With<Caption>>();
    captions.single(app.world()).expect("one caption").0.clone()
}

/// Returns how many nodes are drawn on the sheet of the chart.
pub fn drawn(app: &mut App) -> usize {
    let mut sheets = app
        .world_mut()
        .query_filtered::<Option<&Children>, With<Chart>>();
    let children = sheets.single(app.world()).expect("one sheet");
    children.map_or(0, |children| children.len())
}
