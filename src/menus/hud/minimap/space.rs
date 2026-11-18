//! Drawing the space view: the system chart on its sheet, with the marks of the board
//! computer at the size of the minimap.
//!
//! The chart is drawn again only when a mark moved by a pixel or the destination changed, and
//! it is looked at a few times per second: nothing moves fast on a system chart.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::orbit::{Scene, scene, signature};
use super::pose::Reader;
use super::widget::{Caption, Chart, MARK};
use crate::computer::{
    Atlas, Destination, Dot, SIZE, View, body, frame, ring, ship_position, side, spot,
};
use crate::space::{Clock, Frame};
use crate::ui::{Icon, Tone, icon_sized, theme};
use crate::universe::Universe;

/// Dots of the line that shows the course.
const DASHES: usize = 12;

/// Side in pixels of the frame around the destination.
const CHOSEN: f32 = 12.0;

/// What the corner says where there is no system to chart.
pub const QUIET: &str = "no chart";

/// What the space view reads of the universe.
#[derive(SystemParam)]
pub struct Sky<'w> {
    /// The universe.
    universe: Option<Res<'w, Universe>>,
    /// What the player knows.
    atlas: Option<Res<'w, Atlas>>,
    /// Universe time.
    clock: Option<Res<'w, Clock>>,
    /// The destination of the board computer.
    chosen: Option<Res<'w, Destination>>,
}

impl Sky<'_> {
    /// Returns the chart of the system the point `at` of `frame` is in, or nothing between
    /// the stars.
    fn scene(&self, (frame, at): (Frame, DVec3)) -> Option<Scene> {
        let (universe, atlas) = (self.universe.as_deref()?, self.atlas.as_deref()?);
        let Frame::System(star) = frame else {
            return None;
        };
        let time = self.clock.as_deref().map_or(0.0, |clock| clock.seconds);
        let view = View {
            universe,
            atlas,
            ship: ship_position(universe, frame, at, time)?,
            time,
        };
        let chosen = self.chosen.as_deref().and_then(|chosen| chosen.0);
        Some(scene(&view, star, chosen))
    }
}

/// The nodes the space view writes.
#[derive(SystemParam)]
pub struct Canvas<'w, 's> {
    /// The sheet of the chart.
    sheets: Query<'w, 's, (Entity, &'static mut Node), With<Chart>>,
    /// The sheet of a minimap that was just spawned.
    fresh: Query<'w, 's, (), Added<Chart>>,
    /// The text in the corner.
    captions: Query<'w, 's, &'static mut Text, With<Caption>>,
}

/// Spawns the dotted line of the course from the ship to the destination, and a frame around
/// the destination.
fn course(sheet: &mut ChildSpawnerCommands, scene: &Scene) {
    let Some(to) = scene.chosen.and_then(|row| scene.place(Dot::Row(row))) else {
        return;
    };
    let from = scene.place(Dot::Ship).unwrap_or(to);
    for step in 1..DASHES {
        let at = from.lerp(to, step as f32 / DASHES as f32);
        let node = spot(at, Vec2::splat(-0.5), Vec2::splat(1.0));
        sheet.spawn((node, BackgroundColor(theme::MUTED)));
    }
    frame(sheet, to, CHOSEN);
}

/// Spawns the chart: the orbits, the course, the star and the planets, then the ship over them.
fn draw(sheet: &mut ChildSpawnerCommands, scene: &Scene) {
    let rows = scene.dots.iter().filter_map(|(dot, at)| match dot {
        Dot::Row(index) => Some((*index, *at)),
        _ => None,
    });
    for (_, at) in rows.clone().filter(|(index, _)| *index > 0) {
        sheet.spawn(ring(at.length()));
    }
    course(sheet, scene);
    for (index, at) in rows {
        let line = &scene.lines[index];
        let color = [theme::MUTED, theme::TEXT][usize::from(line.visited)];
        let world = scene.worlds.get(index).and_then(Option::as_ref);
        body(sheet, at, (side(line.target, false), color), world);
    }
    if let Some(at) = scene.place(Dot::Ship) {
        let node = spot(at, Vec2::splat(-0.5 * MARK), Vec2::splat(MARK));
        let symbol = icon_sized(Icon::Send, Tone::Plain, MARK);
        sheet.spawn(node).with_child(symbol);
    }
}

/// Moves and sizes the sheet so that the map shows what the lens of `scene` looks at, or
/// hides it when there is no chart.
fn slide(node: &mut Node, scene: Option<&Scene>) {
    let Some(lens) = scene.map(|scene| scene.lens) else {
        node.display = Display::None;
        return;
    };
    let corner = lens.to_map(Vec2::new(-1.0, 1.0)) / SIZE * 100.0;
    let side = percent(lens.zoom * 100.0);
    node.display = Display::Flex;
    (node.left, node.top) = (percent(corner.x), percent(corner.y));
    (node.width, node.height) = (side, side);
}

/// Returns what the corner says for `scene`: the name of its star, or that there is no chart.
fn title(scene: Option<&Scene>) -> String {
    let named = scene.and_then(|scene| scene.lines.first());
    named.map_or(QUIET.to_string(), |star| star.title.clone())
}

/// Draws the system chart while the player is in space, clears it on a planet, and says so
/// in the corner where there is no system. It only touches the nodes when the picture changed.
pub fn chart(
    mut commands: Commands,
    reader: Reader,
    sky: Sky,
    mut canvas: Canvas,
    mut last: Local<Vec<IVec2>>,
) {
    let grounded = reader.pose().is_some();
    let place = reader.place().filter(|_| !grounded);
    let scene = place.and_then(|at| sky.scene(at));
    let state = vec![IVec2::new(i32::from(grounded), 0)];
    let key = scene.as_ref().map_or(state, signature);
    if *last == key && canvas.fresh.is_empty() {
        return;
    }
    *last = key;
    for (sheet, mut node) in &mut canvas.sheets {
        slide(&mut node, scene.as_ref());
        let mut sheet = commands.entity(sheet);
        sheet.despawn_related::<Children>();
        if let Some(scene) = scene.as_ref() {
            sheet.with_children(|sheet| draw(sheet, scene));
        }
    }
    for mut text in canvas.captions.iter_mut().filter(|_| !grounded) {
        text.0 = title(scene.as_ref());
    }
}
