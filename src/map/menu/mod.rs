//! The menu of a right click on the map: a short list of what can be done with what is under
//! the pointer, opened at the pointer.
//!
//! Bare ground on a planet can take a waypoint or a favourite, give its coordinates or be
//! centred. A planet, a star or a galaxy can be the course or be focused, and a planet can
//! also take a waypoint or a favourite. The waypoint of the player can be the course, be
//! focused, be cleared or be saved. A favourite can be the course, be focused, be renamed or be
//! deleted, and a player can be focused or be the course. A star or a galaxy takes no
//! favourite: a favourite is a place on a planet.
//!
//! The menu closes on Escape, on a press outside it, on a choice and when the map closes.

mod act;
mod show;
mod spot;

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::computer::{Computer, Target};
use crate::ui::{Icon, theme};
use crate::universe::BodyId;

/// Width in pixels of the menu.
pub const WIDTH: f32 = 208.0;

/// Height in pixels of one line of the menu.
pub const ROW: f32 = theme::ROW_COMPACT;

/// Pixels between the edge of the menu and its lines.
pub const PAD: f32 = theme::GAP_XS;

/// The menu of a right click, while it is open.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct ContextMenu(pub Option<Opened>);

/// An open menu: what it is for and the top left corner of it in window pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opened {
    /// What was under the pointer.
    pub under: Under,
    /// The top left corner of the menu in the window.
    pub at: Vec2,
}

/// What can be under the pointer at a right click.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Under {
    /// A place on a planet, as a direction from its centre, with no marker of its own.
    Ground(BodyId, DVec3),
    /// Empty chart, at this chart point.
    Chart(Vec2),
    /// The mark of a planet, a star or a galaxy on the chart.
    Place(Target),
    /// The waypoint of this player, on this body in this direction.
    Mine(BodyId, DVec3),
    /// The favourite at this index of the list.
    Favourite(usize),
    /// The player at this index of the list of friends, on this body.
    Player(usize, BodyId),
}

/// One line of the menu: what it does.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Set the course to it.
    Course,
    /// Go into its level, or centre the map on it.
    Focus,
    /// Set the waypoint of this player there.
    Waypoint,
    /// Clear the waypoint of this player.
    Unwaypoint,
    /// Save it as a favourite and start naming it.
    Favourite,
    /// Copy its latitude and longitude.
    Copy,
    /// Centre the map there.
    Centre,
    /// Type a new name for the favourite.
    Rename,
    /// Forget the favourite.
    Delete,
}

/// Returns the lines of the menu for what is under the pointer, in order.
pub fn choices(under: &Under) -> &'static [Choice] {
    use Choice::*;
    match under {
        Under::Ground(..) => &[Waypoint, Favourite, Copy, Centre],
        Under::Chart(_) => &[Centre],
        Under::Place(Target::Body(_)) => &[Course, Focus, Waypoint, Favourite],
        Under::Place(_) => &[Course, Focus],
        Under::Mine(..) => &[Course, Focus, Unwaypoint, Favourite],
        Under::Favourite(_) => &[Course, Focus, Rename, Delete],
        Under::Player(..) => &[Focus, Course],
    }
}

/// Returns the icon and the words of the line `choice` of the menu for `under`.
pub fn words(choice: Choice, under: &Under) -> (Icon, &'static str) {
    let ground = matches!(under, Under::Ground(..));
    match choice {
        Choice::Course => (Icon::Send, "Set course"),
        Choice::Focus => (Icon::Map, "Focus"),
        Choice::Waypoint => (
            Icon::MapPin,
            ["Set waypoint", "Set waypoint here"][usize::from(ground)],
        ),
        Choice::Unwaypoint => (Icon::Eraser, "Clear waypoint"),
        Choice::Favourite => (Icon::Star, "Save as favourite"),
        Choice::Copy => (Icon::Copy, "Copy coordinates"),
        Choice::Centre => (Icon::Map, "Centre here"),
        Choice::Rename => (Icon::Pencil, "Rename"),
        Choice::Delete => (Icon::Trash, "Delete"),
    }
}

/// Returns the top left corner of a menu of `lines` lines opened at the pointer `at`, moved
/// just enough to stay inside a window of `window` pixels.
pub fn place(at: Vec2, window: Vec2, lines: usize) -> Vec2 {
    let size = Vec2::new(WIDTH, lines as f32 * ROW + 2.0 * PAD);
    at.min(window - size).max(Vec2::ZERO)
}

/// Registers the menu: it opens on a right click on the map, after the clicks of the map have
/// been read, and it is closed whenever the map opens or closes.
pub(super) fn plugin(app: &mut App) {
    let closed = |mut menu: ResMut<ContextMenu>| menu.0 = None;
    app.init_resource::<ContextMenu>()
        .add_observer(act::on_choose)
        .add_systems(OnEnter(Computer::Open), closed)
        .add_systems(OnExit(Computer::Open), closed);
}

/// The systems of the open map that close, open and draw the menu, in that order.
pub(super) fn systems() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    (show::dismiss, show::summon, show::draw).chain()
}

#[cfg(test)]
#[path = "../menu_tests.rs"]
mod tests;
