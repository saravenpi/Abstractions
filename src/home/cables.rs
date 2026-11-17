//! The cables from the container to the station: they drop to the ground, lie on it and climb
//! to the charging dock.

use bevy::prelude::*;

use super::draft::Draft;
use super::paint::DARK;
use super::tune::CENTRE;

/// Half thickness of a cable in metres.
const HALF: f32 = 0.03;

/// How far each cable runs beside the middle one, in metres.
const SHIFTS: [f32; 3] = [-0.3, 0.0, 0.3];

/// Shares of the way from the container to the station at which a cable touches the ground.
const RESTS: [f32; 3] = [0.15, 0.5, 0.85];

/// Draws one cable from `from` to `to`, both in the frame of the base, `shift` metres beside
/// the straight line.
fn cable(draft: &mut Draft, from: Vec3, to: Vec3, shift: f32) {
    let side = (to - from).cross(Vec3::Y).normalize_or_zero() * shift;
    let mut line = vec![from + side];
    for (i, share) in RESTS.into_iter().enumerate() {
        let sway = 1.0 + 0.5 * (i as f32 * 2.1 + shift * 9.0).sin();
        let over = from.lerp(to, share) + side * sway;
        let rest = Vec3::new(over.x, draft.ground(over.xz()), over.z);
        draft.feet.push(rest);
        line.push(rest + Vec3::Y * HALF);
    }
    line.push(to + side * 0.5);
    draft.draw(|soup| {
        for pair in line.windows(2) {
            soup.beam(pair[0], pair[1], HALF, DARK);
        }
    });
}

/// Draws the cables from `from` to `to`, both in the frame of the base.
pub fn build(draft: &mut Draft, from: Vec3, to: Vec3) {
    draft.stand(CENTRE, &[], 0.0);
    for shift in SHIFTS {
        cable(draft, from, to, shift);
    }
}
