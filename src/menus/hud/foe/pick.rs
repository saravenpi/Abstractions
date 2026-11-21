//! Which animals carry a strip: the one fought while the fight lasts, and one more, the living
//! animal the crosshair points at within a short range, or else one standing close by.

use bevy::prelude::*;

/// Farthest an animal can be from the eyes and still carry a strip, in metres.
pub const RANGE: f32 = 13.0;

/// Widest angle between the crosshair and an animal that counts as looking at it, in radians.
pub const CONE: f32 = 0.2;

/// Metres within which an animal carries a strip wherever on screen it stands.
pub const NEAR: f32 = 5.0;

/// Returns the animal among `animals`, each with the point of its body, that the eyes at `eye`
/// looking along the unit `dir` attend to: within [`RANGE`], the one nearest the crosshair inside
/// [`CONE`], else the nearest within [`NEAR`] ahead of the eyes.
pub fn looked(
    animals: impl Iterator<Item = (Entity, Vec3)>,
    (eye, dir): (Vec3, Vec3),
) -> Option<Entity> {
    let score = |at: Vec3| {
        let (to, far) = (at - eye, eye.distance(at));
        let angle = to.normalize_or_zero().dot(dir).clamp(-1.0, 1.0).acos();
        match (far <= RANGE && far > 1e-3, angle <= CONE) {
            (true, true) => Some(angle),
            (true, false) if far <= NEAR && to.dot(dir) > 0.0 => Some(CONE + far),
            _ => None,
        }
    };
    animals
        .filter_map(|(animal, at)| Some((score(at)?, animal)))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, animal)| animal)
}

/// Returns the animals that carry a strip: the one `fought`, then the one `looked` at when it is
/// another. Never more than two.
pub fn labels(fought: Option<Entity>, looked: Option<Entity>) -> [Option<Entity>; 2] {
    [fought, looked.filter(|animal| Some(*animal) != fought)]
}
