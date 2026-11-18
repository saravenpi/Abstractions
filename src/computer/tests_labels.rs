//! Computer tests: the names on the map that give way to each other, the scale bar and the
//! sizes of the marks.

use bevy::math::{Rect, Vec2};

use super::chart::span;
use super::draw::side;
use super::labels::{BAR, GLYPH, MOST_LABELS, keep, priority, room, scale_bar};
use super::layout::LINE_HEIGHT;
use super::*;
use crate::universe::{BodyId, GalaxyId, StarId, Universe};

/// A name takes the room of its characters to the right of its mark, centred on its height.
#[test]
fn a_label_takes_the_room_of_its_characters() {
    let taken = room(Vec2::new(100.0, 50.0), 10.0, 5);
    assert!(
        taken
            .min
            .distance(Vec2::new(110.0, 50.0 - 0.5 * LINE_HEIGHT))
            < 1e-3
    );
    assert!(taken.size().distance(Vec2::new(5.0 * GLYPH, LINE_HEIGHT)) < 1e-3);
    assert!(room(Vec2::ZERO, 0.0, 0).is_empty());
}

/// When two names collide the less important one is hidden, a name that touches nothing is
/// drawn, and a hidden name does not hide the names after it.
#[test]
fn the_lower_priority_label_gives_way() {
    let at = |x: f32, y: f32| room(Vec2::new(x, y), 10.0, 6);
    let labels = [at(0.0, 0.0), at(20.0, 5.0), at(60.0, 5.0), at(0.0, 40.0)];
    assert_eq!(keep(&labels), [true, false, true, true]);
    let swapped = [labels[1], labels[0], labels[2], labels[3]];
    assert_eq!(keep(&swapped), [true, false, false, true]);
    let spread = [at(0.0, 0.0), at(200.0, 0.0), at(150.0, 5.0)];
    assert_eq!(keep(&spread), [true, true, false]);
    let apart = [at(0.0, 0.0), at(0.0, 5.0), at(70.0, 5.0)];
    assert_eq!(keep(&apart), [true, false, true]);
    let ranks = [
        (true, true, false),
        (false, true, true),
        (false, false, true),
    ];
    assert_eq!(
        ranks.map(|(under, chosen, seen)| priority(under, chosen, seen)),
        [0, 1, 2]
    );
    assert_eq!(priority(false, false, false), 3);
    assert!(keep(&[]).is_empty());
}

/// However many names ask for room, no more than the cap are drawn.
#[test]
fn a_crowded_map_draws_a_capped_number_of_labels() {
    let column = |row: usize| room(Vec2::new(0.0, row as f32 * 40.0), 0.0, 4);
    let labels: Vec<Rect> = (0..MOST_LABELS + 20).map(column).collect();
    let kept = keep(&labels);
    assert_eq!(kept.iter().filter(|shown| **shown).count(), MOST_LABELS);
    assert!(kept[..MOST_LABELS].iter().all(|shown| *shown));
}

/// The scale bar is a round distance, 1, 2 or 5 times a power of ten, between two fifths of
/// its longest length and that length, and it follows the zoom.
#[test]
fn the_scale_bar_is_round_and_follows_the_zoom() {
    let cases = [
        (1.0, 50.0),
        (13.0, 1000.0),
        (30.0, 2000.0),
        (1.0e9, 5.0e10),
        (0.02, 1.0),
    ];
    for (scale, metres) in cases {
        let (pixels, round) = scale_bar(scale);
        assert_eq!(round, metres, "{scale}");
        assert!((pixels - (metres / scale) as f32).abs() < 1e-3);
        assert!(
            f64::from(pixels) <= BAR && f64::from(pixels) >= BAR * 0.4,
            "{pixels}"
        );
    }
    let far = scale_bar(1000.0).1;
    assert!(scale_bar(250.0).1 < far && scale_bar(4000.0).1 > far);
}

/// A galaxy chart is drawn to scale and a system chart is not, and a star is drawn larger
/// than a planet or a galaxy, each larger while selected.
#[test]
fn only_far_charts_are_to_scale_and_stars_are_larger() {
    let universe = Universe::new(7);
    let atlas = Atlas::new(&universe);
    let star = universe.home().star;
    let ship = universe.star_position(star).unwrap();
    let (universe, atlas, time) = (&universe, &atlas, 0.0);
    let view = View {
        universe,
        atlas,
        ship,
        time,
    };
    assert_eq!(span(Tab::System, &view.system(star), ship), None);
    let galaxies = view.universe();
    let reach = galaxies
        .iter()
        .map(|line| line.distance)
        .fold(0.0, f64::max);
    let unit = span(Tab::Universe, &galaxies, ship).unwrap();
    assert!(unit > 0.0 && unit < 1.2 * reach, "{unit} against {reach}");
    let (sun, planet) = (
        Target::Star(StarId::default()),
        Target::Body(BodyId::default()),
    );
    let galaxy = Target::Galaxy(GalaxyId::default());
    assert!(side(sun, false) > side(planet, false) && side(sun, true) > side(sun, false));
    assert!(side(planet, true) > side(planet, false) && side(galaxy, true) > side(galaxy, false));
}
