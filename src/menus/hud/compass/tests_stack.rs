//! Tests of the stacking of the marks of the compass: marks that would overlap take rows under the
//! bar, the nearest first, and every strip stays on the bar.

use super::show::Seen;
use super::stack::stack;
use crate::menus::hud::needle::WIDTH;

/// Returns a mark `gap` metres away at `at` pixels along the bar, 120 pixels wide.
fn seen(at: f32, gap: f64) -> Seen {
    Seen {
        index: 0,
        at,
        alpha: 1.0,
        gap,
        row: 0,
        wide: 120.0,
        more: 0,
    }
}

/// Two marks on the same spot stack, the nearer one in the bar.
#[test]
fn marks_that_overlap_take_two_rows_nearest_first() {
    let mut marks = vec![seen(200.0, 90.0), seen(210.0, 30.0)];
    stack(&mut marks);
    assert_eq!(marks.len(), 2);
    assert_eq!((marks[0].gap, marks[0].row), (30.0, 0));
    assert_eq!((marks[1].gap, marks[1].row), (90.0, 1));
}

/// Marks a strip apart share the bar, and one at the very end is pulled inside it.
#[test]
fn marks_apart_share_a_row_and_stay_on_the_bar() {
    let mut marks = vec![seen(0.0, 10.0), seen(WIDTH, 20.0)];
    stack(&mut marks);
    assert!(marks.iter().all(|mark| mark.row == 0));
    assert!(
        marks
            .iter()
            .all(|mark| mark.at - 60.0 >= 0.0 && mark.at + 60.0 <= WIDTH)
    );
}

/// Marks past the third row on the same spot are not drawn, but the last row counts them.
#[test]
fn marks_with_no_free_row_are_counted_on_the_last_row() {
    let mut marks: Vec<Seen> = (0..5).map(|i| seen(240.0, f64::from(i))).collect();
    stack(&mut marks);
    assert_eq!(marks.len(), 3);
    assert_eq!(
        marks.iter().map(|mark| mark.more).collect::<Vec<_>>(),
        [0, 0, 2]
    );
    assert!(marks[2].wide > 120.0);
}
