//! Every number of the home base worth tuning: where it may stand, how it is laid out, how it glows.

use bevy::math::Affine3A;
use bevy::prelude::*;

/// Radius in metres of the ground the base keeps for itself, from its centre.
pub const FOOTPRINT: f64 = 32.0;

/// Distance in metres kept between the footprint and any edge of a cube face of the globe.
pub const EDGE_CLEAR: f64 = 150.0;

/// Distance in metres kept between the footprint and the sea.
pub const SHORE_CLEAR: f32 = 20.0;

/// Lowest ground in metres above sea level anywhere under the footprint.
pub const DRY: f32 = 1.5;

/// Largest difference in metres between the highest and the lowest ground under the footprint:
/// the slope limit of a site.
pub const MAX_RELIEF: f32 = 6.0;

/// Largest difference in metres between the highest and the lowest ground under the landing pad.
/// Where no site within reach keeps both limits, the search allows twice as much.
pub const PAD_RELIEF: f32 = 1.5;

/// Distance in metres from the terrain spawn to the nearest candidate site, the step between
/// candidates and the number of rings searched.
pub const SEARCH: (f32, f32, u32) = (44.0, 16.0, 45);

/// Radius in metres of the circle whose ground must all be of the biome of a taiga or a desert site
/// for that site to lie in the heart of it, where the base is not built.
pub const HEART: f32 = 300.0;

/// Radius in metres of the circle the robot first walks around the base, and the metres of
/// distance a site gives up to have no taiga or desert on it.
pub const HARSH: (f32, f32) = (60.0, 300.0);

/// Latitude in degrees north of the places the base is looked for around, mild on a tilted home
/// world whose pole is its coldest place, and how many of them lie around that circle.
pub const STARTS: (f32, u32) = (32.0, 8);

/// Metres of distance a site may give up for one metre less relief under the footprint, and
/// under the pad.
pub const RELIEF_COST: (f32, f32) = (40.0, 80.0);

/// Distance in metres from the base centre within which the base is spawned, and beyond which
/// it is despawned.
pub const SIGHT: (f64, f64) = (3000.0, 3600.0);

/// Depth in metres every leg goes on below the ground it stands on, so no foot shows a gap.
pub const SINK: f32 = 0.35;

/// Half length of a side of the square landing pad, in metres.
pub const PAD_HALF: f32 = 8.0;

/// Height in metres of the pad surface above the ground it follows.
pub const PAD_LIFT: f32 = 0.05;

/// Distance in metres from the hatch within which the operator notices the robot, and the
/// seconds the lights take to answer.
pub const NOTICE: (f64, f32) = (6.0, 1.2);

/// Distance in metres up to which the beacon keeps its true size, and the largest factor it
/// grows by beyond, so that it stays a visible dot from the air.
pub const BEACON_GROWTH: (f32, f32) = (120.0, 24.0);

/// Where a piece stands on the site: its origin east and south of the base centre, in metres,
/// and its turn around the vertical in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// Origin of the piece, east and south of the base centre.
    pub at: Vec2,
    /// Turn around the vertical. Zero keeps the axes of the piece along those of the base.
    pub yaw: f32,
}

impl Spot {
    /// Returns the pose of the piece when its origin is `height` metres above the base centre.
    pub fn pose(self, height: f32) -> Affine3A {
        let at = Vec3::new(self.at.x, height, self.at.y);
        Affine3A::from_rotation_translation(Quat::from_rotation_y(self.yaw), at)
    }

    /// Returns where the point `local` of the piece, right and back of its origin, lies east
    /// and south of the base centre.
    pub fn point(self, local: Vec2) -> Vec2 {
        self.pose(0.0)
            .transform_point3(Vec3::new(local.x, 0.0, local.y))
            .xz()
    }
}

/// The base centre itself, for what is drawn straight in the axes of the base.
pub const CENTRE: Spot = Spot {
    at: Vec2::ZERO,
    yaw: 0.0,
};

/// The container. Its hatch looks along its own +z.
pub const CONTAINER: Spot = Spot {
    at: Vec2::new(-16.0, 6.0),
    yaw: -0.5,
};

/// The station, beside the container on the side away from the pad.
pub const STATION: Spot = Spot {
    at: Vec2::new(-22.4, 3.1),
    yaw: -0.5,
};

/// The landing pad. A ship parked on it points its nose along the pad's own -z.
pub const PAD: Spot = Spot {
    at: Vec2::new(17.5, -8.5),
    yaw: 0.9,
};

/// Where the robot wakes, right and back of the container origin: in front of the hatch.
pub const WAKE: Vec2 = Vec2::new(-0.9, 9.0);
