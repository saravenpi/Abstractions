//! The opening of the wingsuit off a cliff with a tool in the hands: the hand slings the tool
//! over the back first, and the arms spread once it let go. Shot as the other openings, with the
//! tool in hand or not on each `PROBE glide` line.

use super::*;

/// A run off a cliff with the first swung tool of the hotbar drawn: the slot held over the first
/// two frames, then the robot 160 m over the ground at 8.5 m/s and the jump key pressed at once.
pub fn open_cliff_armed(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(slot) = slot_for(world, false).or_else(|| slot_for(world, true)) else {
        return Act::Fail("no tool in the hotbar: run it after the play stages".into());
    };
    let (centre, side) = match site(world) {
        Ok(found) => found,
        Err(failed) => return failed,
    };
    hold_slot(world, slot, frame);
    if frame == 3 {
        view(world, ViewMode::Third, ViewMode::First);
        let (from, to) = (centre - side * 160.0, centre);
        stand(
            world,
            ground(world, from, 160.0),
            ground(world, to, 160.0),
            -0.2,
        );
        beside(world, side);
    }
    if frame == 5 {
        let mut bodies = world.query_filtered::<&mut Motion, With<Player>>();
        for mut motion in bodies.iter_mut(world) {
            motion.velocity = Quat::from_rotation_y(motion.yaw) * Vec3::new(0.0, 0.0, -8.5);
        }
    }
    opening(world, frame, ("cliff-armed", 6))
}
