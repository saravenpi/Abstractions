//! Bodies grown by `life::body::build` for the motion tests: every plan and archetype over a few
//! masses, habitats and seeds on a home-like world.

use crate::life::body::{Body, archetypes, build};
use crate::life::{Activity, Guild, Habitat, History, Plan, Role, World};

/// Gravity of the home-like test world, metres per second squared.
pub const G: f32 = 9.81;

/// Every animal plan.
const PLANS: [Plan; 5] = [
    Plan::Skeletal,
    Plan::Segmented,
    Plan::Radial,
    Plan::Amorphous,
    Plan::Crystalline,
];

/// One grown test body and what it was grown from.
pub struct Grown {
    /// Its plan.
    pub plan: Plan,
    /// Its archetype within the plan.
    pub archetype: u8,
    /// Its mass in kilograms.
    pub mass: f32,
    /// The body.
    pub body: Body,
}

/// Returns a home-like world.
fn world() -> World {
    World {
        gravity: G,
        pressure: 1.0,
        temperature: 288.0,
        flux: 1.0,
        radius: 6371.0,
        ocean: 0.6,
        star: 5780.0,
    }
}

/// Returns a consumer of `guild` in `habitat` weighing `mass` kilograms.
fn kind(guild: Guild, habitat: Habitat, mass: f32) -> (Role, History) {
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

/// Returns the bodies of every plan and archetype in `habitat`, over three masses and two seeds.
pub fn grown(habitat: Habitat) -> Vec<Grown> {
    let w = world();
    let mut out = Vec::new();
    for plan in PLANS {
        for archetype in 0..archetypes(plan) {
            for (seed, mass) in [(3u64, 0.4f32), (8, 30.0), (21, 400.0)] {
                let guild = [Guild::Grazer, Guild::Hunter][(seed % 2) as usize];
                let (role, history) = kind(guild, habitat, mass);
                let body = build(seed, (plan, archetype), &role, &history, &w);
                out.push(Grown {
                    plan,
                    archetype,
                    mass,
                    body,
                });
            }
        }
    }
    out
}
