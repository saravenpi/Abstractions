//! Tests of the rule table and the budgets: part and triangle caps, at most two accent tissues, thicker legs
//! under strong gravity, no wings without air, rounder bodies in the cold.

use super::{every, kind, world};
use crate::life::body::{Body, CRAWLER_PARTS, Locomotion, MAX_PARTS, PartKind, build, triangles};
use crate::life::{Guild, Habitat, Plan};
use crate::universe::NO_AIR;

#[test]
fn bodies_stay_within_their_budgets() {
    every(|body, form, _| {
        let cap = if form == (Plan::Segmented, 0) {
            CRAWLER_PARTS
        } else {
            MAX_PARTS
        };
        let tris: usize = body.parts.iter().map(triangles).sum();
        assert!(
            body.parts.len() <= cap,
            "{form:?} has {} parts",
            body.parts.len()
        );
        assert!(tris <= 1800, "{form:?} has {tris} triangles");
        assert!(
            body.parts.iter().all(|p| p.tissue <= 2),
            "{form:?} uses more than two accents"
        );
        assert!(
            body.features.iter().all(|f| f.tissue <= 2 && f.size > 0.0),
            "{form:?}"
        );
        assert!(body.length > 0.0 && body.height > 0.0, "{form:?}");
    });
}

#[test]
fn crawlers_have_six_to_fourteen_legs() {
    let (role, history) = kind(Guild::Hunter, Habitat::Ground, 2.0);
    let counts: Vec<usize> = (0..64)
        .map(|s| {
            build(
                s,
                (Plan::Segmented, 0),
                &role,
                &history,
                &world(1.0, 1.0, 288.0),
            )
            .legs
            .len()
        })
        .collect();
    assert!(counts.iter().all(|n| (6..=14).contains(n) && n % 2 == 0));
    assert!(counts.contains(&6) && counts.contains(&14));
}

/// Legged forms checked for gravity-driven limb thickness and reach.
const LEGGED: [(Plan, u8); 11] = [
    (Plan::Skeletal, 0),
    (Plan::Skeletal, 1),
    (Plan::Skeletal, 2),
    (Plan::Skeletal, 4),
    (Plan::Skeletal, 5),
    (Plan::Skeletal, 6),
    (Plan::Skeletal, 7),
    (Plan::Segmented, 2),
    (Plan::Segmented, 3),
    (Plan::Segmented, 4),
    (Plan::Radial, 1),
];

#[test]
fn heavy_worlds_give_thicker_legs() {
    let (role, history) = kind(Guild::Grazer, Habitat::Ground, 120.0);
    for form in LEGGED {
        for seed in 0..16 {
            let thigh = |g: f32| {
                let body = build(seed, form, &role, &history, &world(g, 1.0, 288.0));
                let leg = &body.legs[0];
                (
                    body.parts[usize::from(leg.bones[0])].size.x / body.length,
                    leg.lengths.iter().sum::<f32>() / body.length,
                )
            };
            let (heavy, light) = (thigh(2.0), thigh(0.5));
            assert!(
                heavy.0 > light.0,
                "{form:?} seed {seed}: {heavy:?} against {light:?}"
            );
            assert!(
                heavy.1 < light.1,
                "{form:?} seed {seed}: legs should be longer on a light world"
            );
        }
    }
}

#[test]
fn no_air_means_no_wings() {
    let flyer = |seed: u64, pressure: f32, mass: f32| {
        let (role, history) = kind(Guild::Nectar, Habitat::Air, mass);
        build(
            seed,
            (Plan::Skeletal, 3),
            &role,
            &history,
            &world(1.0, pressure, 288.0),
        )
    };
    for seed in 0..32 {
        let thin = flyer(seed, NO_AIR as f32 * 0.5, 0.5);
        assert!(thin.wings.is_none() && thin.parts.iter().all(|p| p.kind != PartKind::Wing));
        let air = flyer(seed, 1.0, 0.5);
        assert!(air.wings.is_some_and(|w| w.span > 0.0 && w.rate > 0.0));
        assert_eq!(air.locomotion, Locomotion::Fly);
    }
    assert!(
        flyer(1, 1.0, 400.0).wings.is_none(),
        "too heavy to fly in home air"
    );
}

#[test]
fn the_cold_gives_rounder_bodies_with_shorter_tails() {
    let (role, history) = kind(Guild::Grazer, Habitat::Ground, 60.0);
    let body = |kelvin: f32| {
        build(
            5,
            (Plan::Skeletal, 0),
            &role,
            &history,
            &world(1.0, 1.0, kelvin),
        )
    };
    let (cold, hot) = (body(220.0), body(330.0));
    let girth = |b: &Body| b.parts[0].size.x / b.parts[0].size.z;
    assert!(girth(&cold) > girth(&hot));
    let tail = |b: &Body| {
        b.tail
            .as_ref()
            .map_or(0.0, |t| t.lengths.iter().sum::<f32>())
    };
    assert!(tail(&cold) < tail(&hot));
}

#[test]
fn producers_have_an_empty_body() {
    let (mut role, history) = kind(Guild::Producer, Habitat::Ground, 5.0);
    role.level = 1.0;
    let body = build(
        3,
        (Plan::Colonial, 0),
        &role,
        &history,
        &world(1.0, 1.0, 288.0),
    );
    assert!(body.parts.is_empty() && body.legs.is_empty() && body.features.is_empty());
}
