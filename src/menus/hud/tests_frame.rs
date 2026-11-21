//! Tests of the frame corner: a whole body shows nothing, a hurt one lists its worst part first.

use super::frame::{lines, tones};
use crate::condition::{Condition, Part};
use crate::ui::theme;

/// A whole body has no lines.
#[test]
fn a_whole_body_shows_nothing() {
    assert!(lines(&Condition::default()).is_empty());
}

/// A hurt body shows its integrity and one line per hurt part, the worst first.
#[test]
fn hurt_parts_are_listed_worst_first() {
    let mut body = Condition::default();
    body.hurt(Part::LegL, 300.0, true);
    body.hurt(Part::Head, 900.0, true);
    let shown = lines(&body);
    assert_eq!(shown.len(), 3);
    assert!(shown[0].starts_with("Frame "));
    assert_eq!(shown[1], "Head 0%");
    assert!(shown[2].starts_with("Left leg "));
}

/// Each row has the colour of what it reads: a broken arm is in the danger colour and a leg
/// under half in the warning colour while the frame as a whole, still over half, is plain.
#[test]
fn a_row_has_the_colour_of_its_own_health() {
    let mut body = Condition::default();
    body.hurt(Part::ArmL, 540.0, true);
    body.hurt(Part::LegR, 480.0, true);
    body.hurt(Part::Torso, 240.0, true);
    let (shown, colours) = (lines(&body), tones(&body));
    assert_eq!(shown.len(), colours.len());
    assert!(shown[0].starts_with("Frame ") && body.integrity() > 0.5);
    assert_eq!(shown[1], "Left arm 10%");
    assert_eq!(shown[2], "Right leg 40%");
    assert_eq!(shown[3], "Torso 90%");
    assert_eq!(
        colours,
        [theme::TEXT, theme::DANGER, theme::WARN, theme::TEXT]
    );
    assert!(tones(&Condition::default()).is_empty());
}

/// When the whole frame is under a quarter, its own row is in the danger colour.
#[test]
fn a_ruined_frame_reads_as_danger() {
    let mut body = Condition::default();
    for part in Part::ALL {
        body.hurt(part, 1.0e6, true);
    }
    assert_eq!(tones(&body)[0], theme::DANGER);
}

/// The frame rows and the instrument block are children of one column in the bottom-left corner,
/// laid out one under the other, so they can never overlap; neither is placed on its own.
#[test]
fn the_frame_rows_sit_under_the_instruments() {
    use bevy::prelude::*;

    use super::frame::Row;
    use super::panel::Instruments;
    let mut world = World::new();
    let stack = world.spawn(super::stack()).id();
    let mut blocks = world.query_filtered::<(&ChildOf, &Node), With<Instruments>>();
    let (block_parent, block) = blocks.single(&world).expect("one instrument block");
    assert_eq!(block_parent.parent(), stack);
    assert_ne!(block.position_type, PositionType::Absolute);
    let mut rows = world.query_filtered::<(&ChildOf, &Node), With<Row>>();
    let mut stacked = world.query::<&Children>();
    let rows: Vec<_> = rows.iter(&world).collect();
    assert!(!rows.is_empty());
    let order: Vec<Entity> = stacked.get(&world, stack).unwrap().iter().collect();
    assert_eq!(order.len(), 2);
    for (parent, node) in rows {
        assert_eq!(
            world.get::<ChildOf>(parent.parent()).map(ChildOf::parent),
            Some(stack)
        );
        assert_ne!(node.position_type, PositionType::Absolute);
    }
    let column = world.get::<Node>(stack).expect("the stack node");
    assert_eq!(column.flex_direction, FlexDirection::Column);
    assert_eq!(column.left, px(crate::ui::theme::EDGE));
    assert_eq!(column.bottom, px(crate::ui::theme::EDGE));
}
