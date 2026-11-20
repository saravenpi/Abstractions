//! Ground below sea level on each class of planet: dry ground where there is no sea, a liquid to swim in elsewhere.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tangent_anchor;
use crate::gear::Gear;
use crate::gear::tune::REF_GRAVITY;
use crate::player::controller::Motion;
use crate::player::flight::Air;
use crate::player::ground::Ground;
use crate::player::intent::Intent;
use crate::player::lead::View;
use crate::player::spawn::rescue;
use crate::player::walk::{Step, velocity};
use crate::player::{BODY_LENGTH, BODY_RADIUS, above_sea};
use crate::space::Anchor;
use crate::universe::{EARTH_RADIUS, PlanetClass, PlanetRecord, STANDARD_GRAVITY, Universe, Water};
use crate::world::globe::Chart;
use crate::world::{Globe, Terrain};

/// Altitude of the body centre in the basin every class is walked in, in metres.
const DEEP: f32 = -20.0;

/// Returns one hand-built record per planet class with the share of liquid its class really has, and
/// whether that makes a sea.
fn classes() -> [(PlanetRecord, bool); 6] {
    let world = |class, pressure, water, ocean| PlanetRecord {
        class,
        pressure,
        water,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    [
        (world(PlanetClass::Ocean, 1.4, Water::Liquid, 0.92), true),
        (PlanetRecord::TEMPERATE, true),
        (world(PlanetClass::Desert, 0.4, Water::None, 0.0), false),
        (world(PlanetClass::Ice, 0.7, Water::Ice, 0.0), false),
        (world(PlanetClass::Lava, 2.5, Water::Steam, 0.3), true),
        (world(PlanetClass::Barren, 0.0, Water::None, 0.0), false),
    ]
}

/// A body in a basin, stepped by the rules of the controller.
struct Basin {
    /// The movement state.
    motion: Motion,
    /// The flight gear.
    gear: Gear,
    /// The per-frame values of a body centred [`DEEP`] under sea level on the planet.
    step: Step,
}

impl Basin {
    /// Returns a body standing [`DEEP`] under sea level on a planet with `record`.
    fn on(record: PlanetRecord) -> Self {
        let globe = Globe::new(4, 3000.0, STANDARD_GRAVITY).with_record(record);
        let anchor = tangent_anchor(DVec3::Y);
        let view = View {
            yaw: 0.0,
            pitch: 0.0,
            third: false,
        };
        let step = Step {
            dt: 1.0 / 60.0,
            time: 0.0,
            view,
            height: DEEP,
            afloat: above_sea(&anchor, &globe, Vec3::Y * DEEP),
            gravity: REF_GRAVITY,
            air: 1.0,
            pace: 1.0,
            body: Default::default(),
        };
        Self {
            motion: Motion::default(),
            gear: Gear::default(),
            step,
        }
    }

    /// Runs `frames` frames with the forward key held, `jump` held or not, over flat ground or over none.
    fn run(&mut self, frames: usize, jump: bool, floor: Option<Ground>) {
        let intent = Intent {
            wish: Vec3::NEG_Z,
            forward: 1.0,
            right: 0.0,
            sprint: false,
            crouch: false,
            jump,
            yaw: 0.0,
            steer: false,
        };
        let air = Air {
            density: 1.2,
            wind: Vec3::ZERO,
            clearance: 0.0,
            look: Vec3::NEG_Z,
            aim: Vec2::ZERO,
            gravity: REF_GRAVITY,
            liquid: 1000.0,
        };
        for _ in 0..frames {
            let body = (&mut self.motion, &mut self.gear);
            self.motion.velocity = velocity(body, &intent, floor, &air, &self.step);
        }
    }
}

/// On a world without a sea a body deep in a basin walks at full pace, jumps and falls as on any
/// ground and never swims. Where a liquid fills the basin, water or molten rock, it swims up.
#[test]
fn a_dry_basin_is_ground_and_a_filled_one_is_a_sea() {
    let flat = Some(Ground {
        normal: Vec3::Y,
        gap: 0.0,
    });
    for (record, wet) in classes() {
        let mut body = Basin::on(record);
        body.run(180, false, flat);
        let (motion, class) = (body.motion, record.class);
        assert_eq!((motion.swimming, motion.grounded), (wet, !wet), "{class:?}");
        if wet {
            assert!(motion.velocity.y > 0.5 && motion.velocity.length() < 5.0);
            continue;
        }
        assert!((motion.velocity - Vec3::NEG_Z * 5.0).length() < 0.01);
        body.run(1, true, flat);
        assert!(body.motion.velocity.y > 4.0 && !body.motion.grounded);
        let rising = body.motion.velocity.y;
        body.run(30, false, None);
        let falling = body.motion.velocity.y;
        assert!(
            (falling - (rising - REF_GRAVITY * 0.5)).abs() < 0.01,
            "{class:?}"
        );
        assert!(!body.motion.swimming && !body.gear.state.mode.drives());
    }
}

/// Returns the lowest ground of `terrain` on a grid of 60 m over 12 km, as a chart position.
fn lowest(terrain: &Terrain) -> Vec3 {
    let spots = (0..40_000).map(|k| Vec2::new((k % 200) as f32, (k / 200) as f32) * 60.0);
    let ground = spots.map(|at| Vec3::new(at.x, terrain.height(at.x, at.y), at.y));
    ground
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("the grid is not empty")
}

/// Nobody is sent home or lifted for standing in the deepest basin of a dry world. A body under
/// that ground is put back on it, and one that fell out of the world goes home.
#[test]
fn the_deepest_dry_basin_is_no_fall_out_of_the_world() {
    let dry = classes()[5].0;
    let globe = Globe::new(7, EARTH_RADIUS, STANDARD_GRAVITY).with_record(dry);
    let terrain = Terrain::on(Arc::new(globe), Chart::Flat);
    let floor = lowest(&terrain);
    assert!(floor.y < -25.0 && floor.y > -50.0, "floor at {}", floor.y);
    let body = Universe::new(7).home();
    let anchor = Anchor::surface(body, &terrain, floor);
    let home = terrain.embed(0.0, 100.0, 0.0);
    let stand = BODY_LENGTH * 0.5 + BODY_RADIUS;
    let at = |lift: f32| anchor.to_render(terrain.embed(floor.x, floor.y + lift, floor.z));
    assert_eq!(rescue(&anchor, &terrain, at(stand), home), None);
    assert_eq!(rescue(&anchor, &terrain, at(-2.0), home), None);
    let lifted = rescue(&anchor, &terrain, at(-4.0), home).expect("a sunk body is lifted");
    assert!((lifted.y - at(stand).y).abs() < 0.1 && lifted.x == at(stand).x);
    let lost = rescue(&anchor, &terrain, at(-60.0), home);
    assert_eq!(lost, Some(anchor.to_render(home)));
}
