//! Drawing the space view: the system chart on its sheet, with the marks of the board
//! computer at the size of the minimap, moons and rings included, and turning the arrows of
//! the minimap to where the player faces.
//!
//! The chart is drawn again only when a mark moved a little or the destination changed, and the
//! system is looked at a few times per second: the planets are slow. The ship is not: it is
//! placed every frame by [`super::follow`].

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::follow::{Charted, slide};
use super::orbit::{Scene, scene, signature};
use super::pose::Reader;
use super::widget::{Caption, Chart, MARK, Needle, STEP};
use crate::computer::{
    Atlas, Destination, Dot, HOME_LABEL, View, arrow, bearing, body, frame, is_moon, ring,
    satellites, seen, ship_position, side, spot,
};
use crate::space::{Clock, Frame, UniPos};
use crate::ui::theme;
use crate::universe::{StarId, Universe};

/// Dots of the line that shows the course.
pub const DASHES: usize = 12;

/// Marks one dot of the line that shows the course, with its step from the ship.
#[derive(Component, Debug, Clone, Copy)]
pub struct Dash(pub usize);

/// Marks the arrow of the ship on the chart.
#[derive(Component, Debug, Clone, Copy)]
pub struct ShipDot;

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
    /// Returns where the point `at` of `frame` is in the universe.
    pub fn ship(&self, frame: Frame, at: DVec3) -> Option<UniPos> {
        let time = self.clock.as_deref().map_or(0.0, |clock| clock.seconds);
        ship_position(self.universe.as_deref()?, frame, at, time)
    }

    /// Returns the chart of the system of `star` for a ship at the point `at` of `frame`.
    fn scene(&self, star: StarId, (frame, at): (Frame, DVec3)) -> Option<Scene> {
        let (universe, atlas) = (self.universe.as_deref()?, self.atlas.as_deref()?);
        let time = self.clock.as_deref().map_or(0.0, |clock| clock.seconds);
        let view = View {
            universe,
            atlas,
            ship: self.ship(frame, at)?,
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
    /// The chart as it was last looked at, kept for the system that follows the ship.
    charted: ResMut<'w, Charted>,
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
        sheet.spawn((Dash(step), node, BackgroundColor(theme::TEXT_2)));
    }
    frame(sheet, to, (CHOSEN, theme::TEXT));
}

/// Spawns the chart: the orbits, the moons and rings of the planets, the course, the star,
/// the planets and the moons that the zoom parts from their planet, then the ship over them.
fn draw(sheet: &mut ChildSpawnerCommands, scene: &Scene) {
    let (lines, zoom) = (scene.lines.as_slice(), scene.lens.zoom);
    let rows = scene.dots.iter().filter_map(|(dot, at)| match dot {
        Dot::Row(index) => Some((*index, *at)),
        _ => None,
    });
    let planet = |index: usize| index > 0 && !is_moon(lines[index].target);
    for (_, at) in rows.clone().filter(|(index, _)| planet(*index)) {
        sheet.spawn(ring(at.length()));
    }
    satellites(sheet, (&scene.dots, lines, &scene.worlds), zoom);
    course(sheet, scene);
    for (index, at) in rows.filter(|(index, _)| seen(Dot::Row(*index), lines, zoom)) {
        let line = &lines[index];
        let color = [theme::TEXT_2, theme::TEXT][usize::from(line.visited)];
        let world = scene.worlds.get(index).and_then(Option::as_ref);
        body(sheet, at, (side(line.target, false), color), world);
    }
    if let Some(at) = scene.place(Dot::Ship) {
        let node = Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..spot(at, Vec2::splat(-0.5 * MARK), Vec2::splat(MARK))
        };
        sheet
            .spawn((ShipDot, node))
            .with_child((Needle, arrow(STEP)));
    }
}

/// Returns what the corner says for `scene`: the name of its star alone, without the word
/// the board computer adds to the home star, or that there is no chart.
fn title(scene: Option<&Scene>) -> String {
    let named = scene.and_then(|scene| scene.lines.first());
    let bare = |title: &str| title.trim_end_matches(HOME_LABEL).trim_end().to_string();
    named.map_or(QUIET.to_string(), |star| bare(&star.title))
}

/// Turns every arrow of the minimap, every frame, to where the player faces: on the ground by
/// the turn from the view to the body, in space by where the nose points on the chart.
pub fn steer(reader: Reader, mut needles: Query<&mut UiTransform, With<Needle>>) {
    let angle = match reader.pose() {
        Some(pose) => Some(pose.needle),
        None => reader.facing().map(|(nose, top)| bearing(nose, top)),
    };
    let Some(wanted) = angle.map(Rot2::radians) else {
        return;
    };
    for mut needle in &mut needles {
        if needle.rotation != wanted {
            needle.rotation = wanted;
        }
    }
}

/// Draws the system chart again: clears it on a planet, says so in the corner where there is
/// no system, and spawns every mark of `scene` otherwise.
fn redraw(commands: &mut Commands, canvas: &mut Canvas, scene: Option<&Scene>, grounded: bool) {
    for (sheet, mut node) in &mut canvas.sheets {
        slide(&mut node, scene);
        let mut sheet = commands.entity(sheet);
        sheet.despawn_related::<Children>();
        if let Some(scene) = scene {
            sheet.with_children(|sheet| draw(sheet, scene));
        }
    }
    for mut text in canvas.captions.iter_mut().filter(|_| !grounded) {
        text.0 = title(scene);
    }
}

/// Looks at the system while the player is in space, and draws the chart again when a mark
/// moved by a fraction of a pixel, the zoom parted the moons, or the destination changed. The
/// ship and the course are not part of that look: [`super::follow::follow`] moves them every
/// frame. The chart is kept for it.
pub fn chart(
    mut commands: Commands,
    reader: Reader,
    sky: Sky,
    mut canvas: Canvas,
    mut last: Local<Vec<IVec2>>,
) {
    let grounded = reader.pose().is_some();
    let place = reader.place().filter(|_| !grounded);
    let star = place.and_then(|(frame, _)| canvas.charted.star_in(frame));
    let scene = star.zip(place).and_then(|(star, at)| sky.scene(star, at));
    let state = vec![IVec2::new(i32::from(grounded), 0)];
    let key = scene.as_ref().map_or(state, signature);
    if *last != key || !canvas.fresh.is_empty() {
        *last = key;
        redraw(&mut commands, &mut canvas, scene.as_ref(), grounded);
    }
    canvas.charted.scene = scene;
}
