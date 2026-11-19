//! The body over its feet: the ground plane under the feet gives the height, pitch and roll.

use bevy::math::{Quat, Vec3};

use super::feet::Foot;

/// Largest pitch or roll the feet may give the body, in radians.
const TILT: f32 = 0.6;

/// Returns the height of the ground under the body origin `at`, the pitch (nose up) and the roll
/// (right side up) of the plane that best fits where `feet` stand or will land, seen from a body
/// turned by `yaw`.
///
/// The plane is a least-squares fit, steadied so that two feet side by side or in a line give no
/// tilt across the line they make.
pub fn attitude(feet: &[Foot], at: Vec3, yaw: Quat) -> Vec3 {
    if feet.is_empty() {
        return Vec3::new(at.y, 0.0, 0.0);
    }
    let back = yaw.inverse();
    let points: Vec<Vec3> = feet
        .iter()
        .map(|f| back * (f.to - at.with_y(0.0)))
        .collect();
    let mean = points.iter().copied().sum::<Vec3>() / points.len() as f32;
    let (mut xx, mut xz, mut zz, mut xy, mut zy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for p in points.iter().map(|p| *p - mean) {
        (xx, xz, zz) = (xx + p.x * p.x, xz + p.x * p.z, zz + p.z * p.z);
        (xy, zy) = (xy + p.x * p.y, zy + p.z * p.y);
    }
    let ridge = 1e-5 + 0.05 * (xx + zz);
    let (xx, zz) = (xx + ridge, zz + ridge);
    let det = xx * zz - xz * xz;
    let slope_x = (xy * zz - zy * xz) / det;
    let slope_z = (zy * xx - xy * xz) / det;
    let height = mean.y - slope_x * mean.x - slope_z * mean.z;
    Vec3::new(
        height,
        (-slope_z).atan().clamp(-TILT, TILT),
        slope_x.atan().clamp(-TILT, TILT),
    )
}

/// Returns the highest the body origin at `at` may stand so that every planted foot of `feet`
/// stays in reach of its hip, the body turned by `turn` about a point `pivot` metres above its
/// origin; `hips` are each leg's hip in the body frame and its reach. A body with no planted
/// foot has no cap.
pub fn cap(feet: &[Foot], hips: &[(Vec3, f32)], at: Vec3, (turn, pivot): (Quat, f32)) -> f32 {
    let level = at.with_y(0.0);
    let up = Vec3::Y * pivot;
    let reach = |(foot, &(hip, reach)): (&Foot, &(Vec3, f32))| {
        let hip = up + turn * (hip - up);
        let gap = (foot.at - level - hip).with_y(0.0).length();
        foot.at.y + (reach * reach - gap * gap).max(0.0).sqrt() - hip.y
    };
    feet.iter()
        .zip(hips)
        .filter(|(f, _)| f.planted)
        .map(reach)
        .fold(f32::INFINITY, f32::min)
}
