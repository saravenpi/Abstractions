//! Tests of bodies: the same seed gives the same body, parts come parents first, every part off the
//! centre line has a mirrored twin, and the motion model matches the plan.

use bevy::math::Vec3;

use super::{Body, Locomotion, archetypes, build, frames};
use crate::life::{Activity, Guild, Habitat, History, Plan, Role, World};
use crate::universe::hash::Rng;

mod joints;
mod legs;
mod mesh;
mod rules;
mod stature;

/// Every plan.
pub const PLANS: [Plan; 6] = [
    Plan::Skeletal,
    Plan::Segmented,
    Plan::Radial,
    Plan::Amorphous,
    Plan::Colonial,
    Plan::Crystalline,
];

/// Returns a world of `g` home gravities, `pressure` home atmospheres and `temperature` kelvin, home-like
/// otherwise.
pub fn world(g: f32, pressure: f32, temperature: f32) -> World {
    World {
        gravity: g * crate::universe::HOME_GRAVITY as f32,
        pressure,
        temperature,
        flux: 1.0,
        radius: 6000.0,
        ocean: 0.5,
        star: 5800.0,
    }
}

/// Returns a hand-built role and life history of a consumer of `guild` in `habitat` weighing `mass`.
pub fn kind(guild: Guild, habitat: Habitat, mass: f32) -> (Role, History) {
    let role = Role {
        guild,
        habitat,
        activity: Activity::Day,
        niche: 0.5,
        range: 0.2,
        centre: 0.4,
        level: 2.0,
    };
    let history = History {
        mass,
        lifespan: 3600.0,
        clutch: 2,
        group_min: 1,
        group_max: 8,
        care: 0.5,
        speed: 3.0,
        range: 200.0,
    };
    (role, history)
}

/// Returns test case `case`: a world drawn from airless and cold to heavy, hot and small, with a role and
/// a life history drawn over every tested guild, habitat and mass.
fn case(rng: &mut Rng, case: usize) -> (World, Role, History) {
    let guilds = [
        Guild::Grazer,
        Guild::Browser,
        Guild::Hunter,
        Guild::Filter,
        Guild::Nectar,
        Guild::Decomposer,
    ];
    let habitats = [
        Habitat::Ground,
        Habitat::Air,
        Habitat::Water,
        Habitat::Shore,
    ];
    let mut draw = |low: f64, high: f64| rng.range(low, high) as f32;
    let mut w = world(draw(0.2, 3.0), draw(0.0, 4.0), draw(150.0, 360.0));
    w.radius = draw(3000.0, 40000.0);
    w.ocean = draw(0.0, 1.0);
    w.flux = draw(0.05, 3.0);
    w.star = draw(2500.0, 9000.0);
    let mass = 10f32.powf(draw(-2.0, 3.3));
    let (role, history) = kind(guilds[case % 6], habitats[(case / 6) % 4], mass);
    (w, role, history)
}

/// Calls `check` on the body of every plan and archetype over many seeds and test cases.
pub fn every(mut check: impl FnMut(&Body, (Plan, u8), &Role)) {
    let mut rng = Rng::new(7);
    for seed in 0..240 {
        let (w, role, history) = case(&mut rng, seed);
        for plan in PLANS {
            for archetype in 0..archetypes(plan) {
                let body = build(seed as u64, (plan, archetype), &role, &history, &w);
                check(&body, (plan, archetype), &role);
            }
        }
    }
}

#[test]
fn the_same_seed_gives_the_same_body_and_seeds_vary() {
    let (role, history) = kind(Guild::Grazer, Habitat::Ground, 80.0);
    let w = world(1.0, 1.0, 288.0);
    for plan in PLANS {
        for archetype in 0..archetypes(plan) {
            let a = build(11, (plan, archetype), &role, &history, &w);
            assert_eq!(a, build(11, (plan, archetype), &role, &history, &w));
            assert!(!a.parts.is_empty(), "{plan:?} {archetype} has no parts");
        }
    }
    let many: Vec<Body> = (0..8)
        .map(|s| build(s, (Plan::Skeletal, 0), &role, &history, &w))
        .collect();
    assert!(
        many.windows(2).any(|p| p[0] != p[1]),
        "seeds should vary the proportions"
    );
}

#[test]
fn parts_come_parents_first_from_one_root() {
    every(|body, form, _| {
        assert_eq!(body.parts.first().map(|p| p.parent), Some(None), "{form:?}");
        for (i, part) in body.parts.iter().enumerate().skip(1) {
            let parent = part.parent.map(usize::from);
            assert!(
                parent.is_some_and(|p| p < i),
                "{form:?} part {i} before its parent"
            );
        }
    });
}

#[test]
fn every_part_off_the_centre_has_a_mirrored_twin() {
    let flip = Vec3::new(-1.0, 1.0, 1.0);
    let probe = Vec3::new(0.3, -0.7, 0.2);
    every(|body, form, _| {
        let at = frames(&body.parts);
        let point = |i: usize, p: Vec3| at[i].0 + at[i].1 * (p * body.parts[i].size);
        for (i, part) in body.parts.iter().enumerate() {
            let twin = |j: &usize| {
                let other = &body.parts[*j];
                let same = (other.kind, other.shape, other.size, other.tissue)
                    == (part.kind, part.shape, part.size, part.tissue);
                same && point(*j, probe).distance(point(i, probe * flip) * flip)
                    < 1e-3 * body.length.max(0.01)
            };
            match part.side {
                0 => assert!(
                    at[i].0.x.abs() < 1e-4,
                    "{form:?} centre part {i} off the centre line"
                ),
                -1 => assert!(
                    (0..body.parts.len())
                        .filter(|j| body.parts[*j].side == 1)
                        .any(|j| twin(&j)),
                    "{form:?} part {i}"
                ),
                _ => assert!(at[i].0.x > -1e-4, "{form:?} right part {i} on the left"),
            }
        }
        let count = |side: i8| body.parts.iter().filter(|p| p.side == side).count();
        assert_eq!(count(-1), count(1), "{form:?}");
    });
}

#[test]
fn the_motion_model_matches_the_plan() {
    every(|body, (plan, archetype), role| {
        let expected = match plan {
            _ if body.wings.is_some() => Locomotion::Fly,
            Plan::Radial if archetype == 0 => Locomotion::Pulse,
            Plan::Amorphous => Locomotion::Ooze,
            Plan::Colonial => Locomotion::Swarm,
            Plan::Crystalline => Locomotion::Tumble,
            _ if role.habitat == Habitat::Water => Locomotion::Swim,
            Plan::Segmented if archetype == 1 => Locomotion::Slither,
            _ => Locomotion::Walk,
        };
        assert_eq!(body.locomotion, expected, "{plan:?} {archetype}");
        if expected == Locomotion::Walk {
            assert!(
                !body.legs.is_empty(),
                "{plan:?} {archetype} walks without legs"
            );
        }
    });
}
