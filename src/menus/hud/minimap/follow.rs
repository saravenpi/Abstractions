//! The system chart in motion: what the minimap remembers of the chart it drew, and the system
//! that moves the sheet, the ship and the line of the course every frame.
//!
//! Drawing the chart again is slow work and happens a few times a second. The ship is placed
//! on every frame from the position the game uses that frame, so the picture glides with it
//! while the ship flies or warps, and nothing waits for the next look at the system.

use bevy::prelude::*;

use super::orbit::{Scene, lens};
use super::pose::Reader;
use super::space::{DASHES, Dash, ShipDot, Sky};
use super::widget::Chart;
use crate::computer::{Dot, SIZE, ship_dot};
use crate::space::{Frame, UniPos};
use crate::universe::StarId;

/// The chart the minimap last drew and the star it is of.
#[derive(Resource, Default)]
pub struct Charted {
    /// The star of the system on the chart. Between the stars it stays the last one left, so
    /// the chart goes on while the ship leaves the system.
    pub star: Option<StarId>,
    /// The chart, with the ship and the lens kept up to date every frame.
    pub scene: Option<Scene>,
}

impl Charted {
    /// Returns the star to chart for a ship in `frame`, and remembers it: the star of a system
    /// frame, the last one between the stars, none on a planet.
    pub fn star_in(&mut self, frame: Frame) -> Option<StarId> {
        match frame {
            Frame::System(star) => self.star = Some(star),
            Frame::Surface(_) => self.star = None,
            Frame::Galaxy(_) => {}
        }
        self.star
    }
}

/// The nodes that move with the ship.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Moving<'w, 's> {
    /// The sheet of the chart.
    sheets: Query<'w, 's, &'static mut Node, With<Chart>>,
    /// The mark of the ship.
    ships: Query<'w, 's, &'static mut Node, (With<ShipDot>, Without<Chart>)>,
    /// The dots of the course.
    dashes: Query<'w, 's, (&'static Dash, &'static mut Node), (Without<Chart>, Without<ShipDot>)>,
}

/// Returns the `left` and `top` that put a node at the chart point `at`.
fn corner(at: Vec2) -> (Val, Val) {
    let share = (Vec2::new(at.x, -at.y) * 0.5 + 0.5) * 100.0;
    (percent(share.x), percent(share.y))
}

/// Moves `node` to the chart point `at`, touching it only when it moves.
fn put(node: &mut Mut<Node>, at: Vec2) {
    let (left, top) = corner(at);
    if (node.left, node.top) != (left, top) {
        (node.left, node.top) = (left, top);
    }
}

/// Moves and sizes the sheet so that the map shows what the lens of `scene` looks at, or
/// hides it when there is no chart. It touches the node only when something changed.
pub fn slide(node: &mut Mut<Node>, scene: Option<&Scene>) {
    let Some(lens) = scene.map(|scene| scene.lens) else {
        if node.display != Display::None {
            node.display = Display::None;
        }
        return;
    };
    let (left, top) = corner_of(lens.to_map(Vec2::new(-1.0, 1.0)));
    let side = percent(lens.zoom * 100.0);
    let wanted = (Display::Flex, left, top, side, side);
    let now = (node.display, node.left, node.top, node.width, node.height);
    if now != wanted {
        (node.display, node.left, node.top, node.width, node.height) = wanted;
    }
}

/// Returns the `left` and `top` of a corner given in pixels of the map.
fn corner_of(pixel: Vec2) -> (Val, Val) {
    let share = pixel / SIZE * 100.0;
    (percent(share.x), percent(share.y))
}

/// Places the ship of `scene` from `ship`, in its universe position, looks again through the
/// lens, and moves the sheet, the mark of the ship and the dots of the course to match.
fn glide(scene: &mut Scene, ship: UniPos, parts: &mut Moving) {
    let at = ship_dot(&scene.lines, &scene.dots, ship);
    if let Some(slot) = scene.dots.iter_mut().find(|(dot, _)| *dot == Dot::Ship) {
        slot.1 = at;
    }
    scene.lens = lens(&scene.dots, scene.chosen);
    for mut sheet in &mut parts.sheets {
        slide(&mut sheet, Some(scene));
    }
    for mut mark in &mut parts.ships {
        put(&mut mark, at);
    }
    let goal = scene.chosen.and_then(|row| scene.place(Dot::Row(row)));
    for (dash, mut node) in &mut parts.dashes {
        let to = goal.unwrap_or(at);
        put(&mut node, at.lerp(to, dash.0 as f32 / DASHES as f32));
    }
}

/// Follows the ship on the chart, every frame, while the player is in space.
pub fn follow(reader: Reader, sky: Sky, mut charted: ResMut<Charted>, mut parts: Moving) {
    let Some((frame, at)) = reader.place().filter(|_| reader.pose().is_none()) else {
        return;
    };
    let ship = sky.ship(frame, at);
    if let Some((scene, ship)) = charted.scene.as_mut().zip(ship) {
        glide(scene, ship, &mut parts);
    }
}
