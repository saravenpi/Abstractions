//! Where the waves break: the nearest shoreline around the listener, so the sea is heard from
//! where it is and only near it.

use std::f32::consts::TAU;

use bevy::prelude::*;

/// Distances in metres at which the ground around the listener is read, nearest first.
const RINGS: [f32; 8] = [3.0, 6.0, 10.0, 15.0, 21.0, 28.0, 36.0, 45.0];

/// Directions read on each ring.
const SPOKES: usize = 12;

/// Height in metres above the sea at which the shore is no longer heard.
const CEILING: f32 = 45.0;

/// Returns the offset in metres from `at` to the nearest shoreline, on the chart: the middle of
/// the nearest points whose side of the water differs from the one under `at`. Nothing when
/// the ground is all land or all sea within the farthest ring. `height` gives the ground height
/// above the sea at a chart point.
pub fn nearest(height: impl Fn(Vec2) -> f32, at: Vec2) -> Option<Vec2> {
    let wet = height(at) < 0.0;
    RINGS.iter().find_map(|ring| {
        let spoke = |step: usize| Vec2::from_angle(step as f32 * TAU / SPOKES as f32) * *ring;
        let across = (0..SPOKES)
            .map(spoke)
            .filter(|offset| (height(at + *offset) < 0.0) != wet);
        let (sum, count) = across.fold((Vec2::ZERO, 0.0), |(sum, n), offset| {
            (sum + offset, n + 1.0)
        });
        (count > 0.0).then(|| (sum / count).clamp_length_min(RINGS[0]))
    })
}

/// Returns how loud the shore is for a listener `distance` metres from it and `above` metres
/// over the sea: full at the water's edge, gone at the farthest ring and high in the air.
pub fn closeness(distance: f32, above: f32) -> f32 {
    let near = 1.0 - (distance / RINGS[RINGS.len() - 1]).clamp(0.0, 1.0);
    near * near * (1.0 - above / CEILING).clamp(0.0, 1.0)
}

/// Returns the left and right gains of a sound of `level` that comes from `side`, from -1 at
/// the left to 1 at the right. A sound straight ahead keeps its level on both sides, and one
/// fully to a side is never silent on the other, as with two ears.
pub fn stereo(side: f32, level: f32) -> [f32; 2] {
    let lean = side.clamp(-1.0, 1.0) * 0.85;
    [level * (1.0 - lean).sqrt(), level * (1.0 + lean).sqrt()]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A beach to the east is found to the east at its distance, inland nothing is heard, and
    /// a sound from the right is louder on the right.
    #[test]
    fn the_shore_is_where_the_water_meets_the_land() {
        let beach = |at: Vec2| 12.0 - at.x;
        let found = nearest(beach, Vec2::ZERO).unwrap();
        assert!(found.x > 10.0 && found.x < 21.0 && found.y.abs() < 3.0);
        assert_eq!(nearest(beach, Vec2::new(-80.0, 0.0)), None);
        assert_eq!(nearest(|_| 2.0, Vec2::ZERO), None);
        assert!(nearest(beach, Vec2::new(30.0, 0.0)).unwrap().x < 0.0);
        assert!(closeness(3.0, 1.0) > 0.8 && closeness(45.0, 1.0) == 0.0);
        assert!(closeness(3.0, 60.0) == 0.0);
        let [left, right] = stereo(1.0, 1.0);
        assert!(right > 1.3 && left > 0.3 && left < 0.5);
        assert_eq!(stereo(0.0, 0.5), [0.5, 0.5]);
    }
}
