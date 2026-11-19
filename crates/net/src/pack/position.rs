//! Positions on a body in whole millimetres: three `i32` instead of three `f64`.

use glam::DVec3;

/// Farthest a packed position can lie from the origin on each axis, in metres: about 2147 km.
/// A coordinate beyond it is clamped to it.
pub const MM_RANGE: f64 = i32::MAX as f64 / 1000.0;

/// Returns `position`, in metres, in whole millimetres per axis. The error is at most 0.5 mm
/// inside [`MM_RANGE`]; beyond it the coordinate is clamped, and one that is not a number is 0.
pub fn pack_mm(position: DVec3) -> [i32; 3] {
    position
        .to_array()
        .map(|metres| (metres * 1000.0).round() as i32)
}

/// Returns the position in metres that [`pack_mm`] stored.
pub fn unpack_mm(packed: [i32; 3]) -> DVec3 {
    DVec3::from_array(packed.map(|mm| f64::from(mm) / 1000.0))
}
