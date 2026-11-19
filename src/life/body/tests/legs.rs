//! Tests of the limb data the motion model relies on: bone and chain lengths equal the socket distances,
//! feet rest on the ground within reach of the hip, and the rest pose puts each foot on its rest point.

use super::every;
use crate::life::body::{Body, Chain, frame, frames};

/// Checks that `lengths` are the socket distances of `parts` after the first, then the last part's length,
/// and that each part is the child of the one before.
fn follows(body: &Body, parts: &[u8], lengths: &[f32]) {
    assert_eq!(parts.len(), lengths.len());
    let part = |i: usize| &body.parts[usize::from(parts[i])];
    for i in 1..parts.len() {
        assert_eq!(part(i).parent, Some(parts[i - 1]));
        assert!((part(i).socket.length() - lengths[i - 1]).abs() < 1e-5);
    }
    let last = part(parts.len() - 1);
    assert!((last.size.y - lengths[lengths.len() - 1]).abs() < 1e-5);
}

#[test]
fn leg_lengths_match_the_sockets() {
    every(|body, form, _| {
        for leg in &body.legs {
            assert!((2..=3).contains(&leg.bones.len()), "{form:?}");
            follows(body, &leg.bones, &leg.lengths);
        }
    });
}

#[test]
fn feet_rest_on_the_ground_within_reach_of_the_hip() {
    every(|body, form, _| {
        for leg in &body.legs {
            let hip = frame(&body.parts, usize::from(leg.bones[0])).0;
            let total: f32 = leg.lengths.iter().sum();
            let reach = hip.distance(leg.rest) / total;
            assert_eq!(leg.rest.y, 0.0, "{form:?}");
            assert!((0.6..=0.95).contains(&reach), "{form:?} reach {reach}");
            assert!(
                (hip.y - body.height).abs() <= 1e-4 * body.height.max(1.0),
                "{form:?} hip {hip} height {}",
                body.height
            );
        }
    });
}

#[test]
fn the_rest_pose_puts_every_foot_on_its_rest_point() {
    every(|body, form, _| {
        let at = frames(&body.parts);
        for leg in &body.legs {
            let last = usize::from(*leg.bones.last().unwrap_or(&0));
            let tip = at[last].0 + at[last].1 * (bevy::math::Vec3::NEG_Y * body.parts[last].size.y);
            let total: f32 = leg.lengths.iter().sum();
            assert!(
                tip.distance(leg.rest) < 1e-3 * total,
                "{form:?} foot {tip} rest {}",
                leg.rest
            );
        }
    });
}

#[test]
fn legs_go_front_to_back_left_before_right() {
    every(|body, form, _| {
        for pair in body.legs.chunks(2) {
            assert_eq!(pair.len(), 2, "{form:?}");
            assert_eq!((pair[0].side, pair[1].side), (-1, 1), "{form:?}");
            assert_eq!(pair[0].rank, pair[1].rank, "{form:?}");
        }
        assert!(
            body.legs.windows(2).all(|w| w[0].rank <= w[1].rank),
            "{form:?}"
        );
    });
}

#[test]
fn chains_match_the_sockets_and_hang() {
    every(|body, form, _| {
        let chains: Vec<&Chain> = [&body.spine, &body.neck, &body.tail]
            .into_iter()
            .flatten()
            .chain(&body.tentacles)
            .collect();
        for chain in chains {
            assert!(!chain.parts.is_empty(), "{form:?}");
            follows(body, &chain.parts, &chain.lengths);
            assert!(
                chain
                    .parts
                    .iter()
                    .all(|&p| body.parts[usize::from(p)].shape.hangs()),
                "{form:?}"
            );
        }
    });
}
