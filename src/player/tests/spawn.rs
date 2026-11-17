//! Where a new or resumed body starts.

use bevy::prelude::*;

use crate::player::spawn::*;
use crate::saves::WorldSave;
use crate::world::SpawnPoint;

/// A resumed world stands on the spawn point with its saved angles; a new one drops in.
#[test]
fn resumed_pose_keeps_spot_and_angles() {
    let spawn = SpawnPoint(Vec3::new(1.0, 2.0, 3.0));
    let fresh = start_pose(spawn, None);
    assert_eq!(fresh.position, body_position(spawn));
    let save = WorldSave {
        position: Some([5.0, 6.0, 7.0]),
        yaw: 1.0,
        pitch: -0.5,
        ..WorldSave::default()
    };
    let pose = start_pose(spawn, Some(&save));
    assert_eq!(pose.position, Vec3::new(1.0, 2.0 + RESUME_LIFT, 3.0));
    assert_eq!((pose.yaw, pose.pitch), (1.0, -0.5));
    let unplayed = WorldSave::default();
    assert_eq!(start_pose(spawn, Some(&unplayed)), fresh);
}
