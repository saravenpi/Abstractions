//! The flat map of a planet as it is looked at: its lens and what the pointer does on it, the
//! lens easing to its goal, and the sheet the planet is drawn on following the lens.

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::level::{Level, first};
use super::me::Me;
use super::planet::Tile;
use super::sight::Sight;
use crate::computer::{Hand, SIZE};

/// How fast the lens closes the gap to its goal, per second, as on the charts.
const EASE: f32 = 12.0;

/// How the flat map of the planet is looked at and what the pointer does on it.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Surface {
    /// The zoom and the centre of the map as it is drawn.
    pub lens: Sight,
    /// The zoom and the centre the map eases to.
    pub goal: Sight,
    /// True while the pointer is over the map.
    pub over: bool,
    /// The pixel of the map the pointer is at, counted as the lens counts them.
    pub at: Vec2,
    /// Where the pointer was last seen with the left button held, since it went down.
    pub held: Option<Vec2>,
    /// True once the held pointer moved far enough to drag the map.
    pub dragged: bool,
    /// What the last click was on, a marker or nothing, and when in seconds.
    pub clicked: Option<(Option<usize>, f32)>,
    /// The marker under the pointer, by its place in the list of drawn markers.
    pub hover: Option<usize>,
    /// The direction from the centre of the planet under the pointer, on the planet.
    pub point: Option<DVec3>,
    /// The direction the last click on the planet picked.
    pub picked: Option<DVec3>,
    /// The direction the next planet opened is to be looked at, when one was asked for.
    pub focus: Option<DVec3>,
    /// The selected marker, by its place in the list of drawn markers.
    pub chosen: Option<usize>,
    /// The lens the next planet opened is looked through, when Back returns to it.
    pub again: Option<Sight>,
}

/// Marks the sheet the flat map is drawn on, which the lens moves and sizes.
#[derive(Component)]
pub struct PlanetSheet;

/// Looks at the planet anew when the map opens one: where a list asked, on the player on it,
/// or at the whole planet, or as it was looked at when Back returns to it. Any level shown anew
/// starts still.
pub(super) fn enter(
    level: Res<Level>,
    me: Me,
    mut surface: ResMut<Surface>,
    mut hand: ResMut<Hand>,
) {
    hand.drift = Vec2::ZERO;
    let Level::Planet(body) = *level else {
        return;
    };
    let here = me.spot().filter(|(on, _)| *on == body).map(|(_, at)| at);
    let lens = surface
        .again
        .take()
        .unwrap_or_else(|| first(surface.focus.take(), here));
    *surface = Surface {
        lens,
        goal: lens,
        ..default()
    };
}

/// Moves the lens part of the way to its goal, so a zoom or a jump glides instead of cutting.
pub(super) fn ease(time: Res<Time>, mut surface: ResMut<Surface>) {
    if surface.lens == surface.goal {
        return;
    }
    let share = 1.0 - (-EASE * time.delta_secs()).exp();
    surface.lens = surface.lens.toward(surface.goal, share);
}

/// Moves and sizes the sheet of the planet so the map shows what the lens looks at, and puts
/// the copies of the picture around the turn of longitude under the lens.
pub(super) fn slide(
    surface: Res<Surface>,
    mut sheets: Query<&mut Node, With<PlanetSheet>>,
    mut tiles: Query<(&Tile, &mut Node), Without<PlanetSheet>>,
) {
    let corner = surface.lens.to_map(DVec2::new(-1.0, 1.0)) / SIZE * 100.0;
    let side = percent(surface.lens.zoom * 100.0);
    let wanted = (percent(corner.x), percent(corner.y), side, side);
    for mut node in &mut sheets {
        if (node.left, node.top, node.width, node.height) != wanted {
            (node.left, node.top, node.width, node.height) = wanted;
        }
    }
    let turn = (surface.lens.centre.x * 0.5).round() as f32;
    for (Tile(east), mut node) in &mut tiles {
        let left = percent((turn + f32::from(*east)) * 100.0);
        if node.left != left {
            node.left = left;
        }
    }
}
