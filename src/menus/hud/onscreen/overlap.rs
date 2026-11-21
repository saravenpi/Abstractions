//! Markers that meet on the screen: the one that ranks first keeps its room, the next is
//! reduced to a small icon without words, and one that would still print over another is left
//! out. Markers are asked in rank order, so the course always wins, then friends, home, the
//! ship and pins.

use bevy::prelude::*;

use super::rule::Face;
use super::tune::REDUCED_ALPHA;
use crate::ui::theme;

/// Half the width and the height in pixels of a whole marker, icon over words, with the gap
/// that keeps two apart, and how far under the centre of the icon its middle lies.
const WHOLE: (Vec2, f32) = (Vec2::new(48.0, 28.0), 12.0);

/// Half the side in pixels of a small icon with the gap that keeps two apart.
const SMALL: Vec2 = Vec2::splat(theme::ICON_SMALL * 0.5 + theme::S1);

/// Returns the room a marker drawn as `face` with its icon centred on `pos` takes on the screen.
fn room(pos: Vec2, face: Face) -> Rect {
    match face.label > 0.0 {
        true => Rect::from_center_half_size(pos + Vec2::Y * WHOLE.1, WHOLE.0),
        false => Rect::from_center_half_size(pos, SMALL),
    }
}

/// Returns whether `room` meets one of `taken`.
fn meets(room: Rect, taken: &[Rect]) -> bool {
    taken.iter().any(|other| !room.intersect(*other).is_empty())
}

/// Returns `face` reduced: a small icon without words.
fn reduced(face: Face) -> Face {
    Face {
        glyph: face.glyph.min(REDUCED_ALPHA),
        label: 0.0,
        small: true,
    }
}

/// Returns how the marker that wants to be drawn as `face` at `pos` is drawn beside the rooms
/// `taken` by the markers that rank before it, and takes its own room: whole when it is free,
/// reduced when it would meet one, nothing when even its small icon would. A marker that is
/// not drawn takes no room.
pub fn settle(face: Face, pos: Vec2, taken: &mut Vec<Rect>) -> Option<Face> {
    if face.glyph <= 0.0 && face.label <= 0.0 {
        return None;
    }
    let small = reduced(face);
    let shows = |face: &Face| face.glyph > 0.0 || face.label > 0.0;
    let fits = |face: &Face| shows(face) && !meets(room(pos, *face), taken);
    let drawn = [face, small].into_iter().find(fits)?;
    taken.push(room(pos, drawn));
    Some(drawn)
}
