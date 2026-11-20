//! The distance column of the results: every drawn row writes its distance in one unit, the one
//! the farthest needs, so the rows compare at a glance.

use super::board::Board;
use super::rows::Line;
use super::show::cell_text;
use super::view::Part;
use crate::ui::Tone;
use crate::ui::units::distances_short;

/// Returns the distances of the drawn rows, all in the unit the farthest needs.
pub(super) fn shared(board: &Board) -> Vec<String> {
    let gaps: Vec<f64> = (0..board.window())
        .filter_map(|slot| board.at(slot))
        .map(|line| line.distance)
        .collect();
    distances_short(&gaps)
}

/// Returns the text and the tone of one cell of `line` at `slot`, the distance cell in the
/// unit its `shared` list uses.
pub(super) fn cell(
    line: &Line,
    (part, slot): (Part, usize),
    marks: (bool, bool),
    shared: &[String],
) -> (String, Tone) {
    let (shown, tone) = cell_text(line, part, marks);
    match (part, shared.get(slot)) {
        (Part::Distance, Some(own)) => (own.clone(), tone),
        _ => (shown, tone),
    }
}
