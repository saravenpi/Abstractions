//! The puffs the body kicks up: thrown by real events only, made of the real ground, and
//! flying as the air of the planet lets them.

use bevy::prelude::*;

use crate::avatar::dust::{Burst, From, Seen, Soil, Way, bursts, landing, soil};
use crate::avatar::marks::Marks;
use crate::avatar::puffs::{Puff, Thrower, drift, throw};
use crate::avatar::{AvatarPose, Mode};
use crate::universe::PlanetClass;
use crate::world::Surface;

/// A frame at sixty frames per second.
const DT: f32 = 1.0 / 60.0;

/// Returns the bursts thrown between a body in `was` and the same body in `now`.
fn thrown(was: &AvatarPose, now: &AvatarPose, planted: [[bool; 2]; 2]) -> [Option<Burst>; 5] {
    bursts(
        &Seen::of(was, planted[0]),
        &Seen::of(now, planted[1]),
        now,
        DT,
    )
}

/// Returns a state in `mode` at ground speed `speed`.
fn state(mode: Mode, speed: f32) -> AvatarPose {
    AvatarPose {
        mode,
        speed,
        ..default()
    }
}

/// Snow, sand, grass, rock and the bare ground of a lava world all raise something different,
/// any liquid raises a splash, and every soil has its own colour.
#[test]
fn the_ground_under_the_feet_chooses_the_puff() {
    let home = PlanetClass::Island;
    let dry = [Surface::Snow, Surface::Sand, Surface::Grass, Surface::Rock].map(|s| soil(home, s));
    assert_eq!(dry, [Soil::Snow, Soil::Sand, Soil::Earth, Soil::Rock]);
    assert_eq!(soil(PlanetClass::Lava, Surface::Sand), Soil::Land);
    assert_eq!(soil(PlanetClass::Lava, Surface::Snow), Soil::Snow);
    for wet in [Surface::Water, Surface::Shallows, Surface::Pond] {
        assert_eq!(soil(home, wet), Soil::Splash);
        assert_eq!(soil(PlanetClass::Lava, wet), Soil::Splash);
    }
    let tints = Soil::ALL.map(|soil| soil.tint(PlanetClass::Lava));
    for (index, tint) in tints.iter().enumerate() {
        assert!(tint.iter().all(|c| (0.0..=1.0).contains(c)), "{tint:?}");
        assert!(!tints[..index].contains(tint), "two soils look the same");
    }
    assert_ne!(
        Soil::Land.tint(PlanetClass::Lava),
        Soil::Land.tint(PlanetClass::Ice)
    );
    assert_ne!(
        Soil::Splash.tint(PlanetClass::Lava),
        Soil::Splash.tint(home)
    );
}

/// A touch-down throws a ring sized by the speed really lost, a step down next to nothing,
/// and a body that stays in the air or stays on the ground throws none.
#[test]
fn a_landing_throws_a_ring_sized_by_its_speed() {
    let (soft, hard) = (landing(4.0), landing(16.0));
    assert_eq!(landing(0.5).count, 0.0);
    assert!(hard.count > soft.count && soft.count >= 1.0, "{soft:?}");
    assert!(hard.speed > soft.speed && hard.size > soft.size && hard.life > soft.life);
    assert_eq!((hard.from, hard.way), (From::Feet, Way::Ring));
    let falling = AvatarPose {
        vertical_speed: -9.0,
        ..state(Mode::Fall, 0.0)
    };
    let landed = AvatarPose {
        absorb: true,
        impact: 9.0,
        ..state(Mode::Land, 0.0)
    };
    let none = [[false; 2]; 2];
    assert_eq!(thrown(&falling, &landed, none)[0], Some(landing(9.0)));
    assert_eq!(thrown(&falling, &falling, none), [None; 5]);
    assert_eq!(thrown(&landed, &landed, none), [None; 5]);
    let swum = state(Mode::Swim, 0.0);
    assert_eq!(thrown(&falling, &swum, none)[0], Some(landing(9.0)));
    assert_eq!(thrown(&swum, &swum, none), [None; 5]);
}

/// A foot that plants at a run raises a puff, a walk raises none, a hard stop skids, a jump
/// kicks dust back, and standing still raises nothing at all.
#[test]
fn steps_skids_and_jumps_raise_dust() {
    let (run, walk) = (state(Mode::Run, 8.5), state(Mode::Walk, 4.0));
    let plants = [[false, true], [true, true]];
    let step = thrown(&run, &run, plants)[1].expect("a step");
    assert_eq!(
        (step.from, step.way, step.count),
        (From::Foot(0), Way::Back, 1.0)
    );
    assert_eq!(thrown(&walk, &walk, plants), [None; 5]);
    assert_eq!(thrown(&run, &run, [[true; 2]; 2]), [None; 5]);
    let braking = AvatarPose {
        gain: -30.0,
        ..state(Mode::Run, 7.0)
    };
    let skid = thrown(&braking, &braking, [[true; 2]; 2])[2].expect("a skid");
    assert!(skid.count > 0.2 && skid.way == Way::Ahead, "{skid:?}");
    let easing = AvatarPose {
        gain: -6.0,
        ..state(Mode::Run, 7.0)
    };
    assert_eq!(thrown(&easing, &easing, [[true; 2]; 2])[2], None);
    let jump = thrown(&run, &state(Mode::Jump, 8.5), [[true; 2]; 2])[3].expect("a kick");
    assert_eq!((jump.from, jump.way), (From::Feet, Way::Back));
    let still = state(Mode::Idle, 0.0);
    assert_eq!(thrown(&still, &still, [[true; 2]; 2]), [None; 5]);
}

/// Wings that spread in moving air show the air they catch, more the faster it moves; at rest,
/// folding, or already spread they show nothing.
#[test]
fn spreading_wings_catch_the_air_only_at_speed() {
    let wings = |open: f32, airspeed: f32| AvatarPose {
        wings: open,
        airspeed,
        ..state(Mode::Glide, 0.0)
    };
    let none = [[false; 2]; 2];
    let caught = |from: f32, to: f32, airspeed: f32| {
        thrown(&wings(from, airspeed), &wings(to, airspeed), none)[4]
    };
    let (slow, fast) = (caught(0.2, 0.3, 14.0), caught(0.2, 0.3, 40.0));
    assert!(fast.expect("air").count > slow.expect("air").count);
    assert_eq!(fast.expect("air").from, From::Tips);
    assert_eq!(caught(0.2, 0.3, 0.0), None);
    assert_eq!(caught(0.3, 0.2, 40.0), None);
    assert_eq!(caught(1.0, 1.0, 40.0), None);
}

/// A puff leaves from the soles, starts small, swells, and shrinks to nothing: it never pops.
/// In air it slows down and hangs; with no air nothing slows it and it falls its whole arc.
#[test]
fn puffs_fade_out_and_fly_as_the_air_lets_them() {
    let thrower = Thrower {
        pose: Transform::from_xyz(5.0, 2.0, -3.0),
        velocity: Vec3::NEG_Z * 6.0,
        up: Vec3::Y,
    };
    let marks = Marks::default();
    let fly = |air: f32| {
        let (mut puff, mut at): (Puff, Vec3) = throw(&landing(12.0), 7, &thrower, &marks);
        let (start, speed) = (at, puff.velocity.with_y(0.0).length());
        assert!(start.distance(thrower.pose.translation) < 1.2 && start.y < 1.3);
        let mut sizes = Vec::new();
        while puff.age < puff.life {
            sizes.push(drift(&mut puff, &mut at, (Vec3::Y, 9.8, air), DT));
        }
        let kept = puff.velocity.with_y(0.0).length() / speed;
        (sizes, kept, at.y - start.y)
    };
    let (sizes, kept, _) = fly(1.0);
    assert!(sizes[0] < 0.5 * puff_peak(&sizes), "it pops in");
    assert!(*sizes.last().expect("a life") < 1e-3, "it pops out");
    let step = sizes
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max);
    assert!(step < 0.08, "its size steps by {step} m");
    assert!(kept < 0.3, "air kept {kept} of its speed");
    let (_, bare, fell) = fly(0.0);
    assert!((bare - 1.0).abs() < 1e-4, "no air kept {bare} of its speed");
    assert!(fell < -0.5, "with no air it fell {fell} m");
}

/// Returns the largest size a puff reached.
fn puff_peak(sizes: &[f32]) -> f32 {
    sizes.iter().copied().fold(0.0, f32::max)
}
