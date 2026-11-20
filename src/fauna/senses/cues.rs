//! The cues an animal notices a threat by: sight over the ground, the noise the threat makes, and
//! its scent carried downwind.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::tune::{BECALMED, BOOTS, EAR, LANDING, RAY_STEPS, SCENT_HALF, WALK};

/// Returns true when the line from `eye` to `target`, both in the animal's frame, clears the ground
/// read through `ground`: a few points along it, each above the ground under it.
pub fn clear(ground: &dyn Fn(Vec2) -> f32, eye: Vec3, target: Vec3) -> bool {
    (1..RAY_STEPS).all(|k| {
        let p = eye.lerp(target, k as f32 / RAY_STEPS as f32);
        ground(p.xz()) < p.y
    })
}

/// Returns how loud a robot is: 1 at a walk, more running, with its boots at `thrust` 0 to 1, and
/// at a landing `impact` metres per second lost into the ground.
pub fn loudness(speed: f32, thrust: f32, impact: f32) -> f32 {
    speed / WALK + BOOTS * thrust + LANDING * impact
}

/// Returns true when a threat of loudness `loud` is heard from `metres` away through ears of range
/// factor `keen`.
pub fn heard(loud: f32, metres: f32, keen: f32) -> bool {
    metres < EAR * loud * keen
}

/// Returns true when the scent of a threat at `off` from the animal, `(x, z)` in its frame, reaches
/// it on a `wind` blowing at `(x, z)` metres per second, through a nose of `nose` metres: the animal
/// lies downwind, within [`SCENT_HALF`] degrees of the wind's line from the threat.
pub fn smelt(wind: Vec2, off: Vec2, nose: f32) -> bool {
    let metres = off.length();
    if wind.length() < BECALMED || metres > nose || metres < 1e-3 {
        return false;
    }
    let down = -off / metres;
    down.dot(wind.normalize()) >= SCENT_HALF.to_radians().cos()
}
