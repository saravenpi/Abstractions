//! Rotations in 32 bits: the three smallest components of a unit quaternion, 10 bits each.
//!
//! The largest component is dropped and found again from the other three, because a unit
//! quaternion has length one. Two bits say which one was dropped. The error is under 0.005 rad.

use glam::Quat;

/// Largest value a kept component can have: one over the square root of two.
const LIMIT: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// Steps a kept component is cut into. An even count, one less than 10 bits allow, so that
/// zero has a step of its own and a turn about one axis comes back exactly.
const STEPS: f32 = 1022.0;

/// Returns the index of the component of `parts` that is largest in size.
fn largest(parts: [f32; 4]) -> usize {
    (1..4).fold(0, |best, i| {
        if parts[i].abs() > parts[best].abs() {
            i
        } else {
            best
        }
    })
}

/// Returns `rotation` in 32 bits: the index of the dropped component in the two high bits,
/// then the three others in order, 10 bits each. A rotation of no length packs as no turn.
pub fn pack_rotation(rotation: Quat) -> u32 {
    let length = rotation.length();
    let sound = length.is_finite() && length > 1e-6;
    let parts = if sound {
        rotation / length
    } else {
        Quat::IDENTITY
    }
    .to_array();
    let dropped = largest(parts);
    let sign = if parts[dropped] < 0.0 { -1.0 } else { 1.0 };
    let kept = (0..4).filter(|&i| i != dropped);
    kept.fold(dropped as u32, |bits, i| {
        let share = (parts[i] * sign / LIMIT).clamp(-1.0, 1.0) * 0.5 + 0.5;
        (bits << 10) | (share * STEPS).round() as u32
    })
}

/// Returns the unit rotation that [`pack_rotation`] stored in `bits`.
pub fn unpack_rotation(bits: u32) -> Quat {
    let dropped = (bits >> 30) as usize;
    let mut parts = [0.0; 4];
    let mut sum = 0.0;
    for (slot, i) in (0..4).filter(|&i| i != dropped).enumerate() {
        let field = (bits >> (20 - 10 * slot)) & 0x3ff;
        parts[i] = (field as f32 / STEPS * 2.0 - 1.0) * LIMIT;
        sum += parts[i] * parts[i];
    }
    parts[dropped] = (1.0 - sum).max(0.0).sqrt();
    Quat::from_array(parts).normalize()
}
