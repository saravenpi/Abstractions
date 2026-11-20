//! Tests of stature: a quadruped grazer stands on legs that grow as the cube root of its mass, about
//! 0.19 m per cube-root kilogram at the hip on a world of home gravity, as mammals do.

use super::{kind, world};
use crate::life::body::{Locomotion, build};
use crate::life::{Guild, Habitat, Plan};

/// Hip height over the cube root of the mass, metres per cube-root kilogram, of large mammals: a
/// 70 kg deer stands about 0.8 m, a 500 kg horse about 1.5 m.
const MAMMAL: f32 = 0.19;

/// Quadruped grazers of 10 kg to 2 t on a home-like world stand within a quarter of the mammal hip
/// height for their mass, and a 75 kg one at least 0.7 m.
#[test]
fn hip_height_grows_as_the_cube_root_of_mass() {
    let w = world(1.0, 1.0, 288.0);
    for mass in [10.0, 30.0, 75.0, 200.0, 500.0, 2000.0_f32] {
        let (role, history) = kind(Guild::Grazer, Habitat::Ground, mass);
        for seed in 0..8 {
            let body = build(seed, (Plan::Skeletal, 0), &role, &history, &w);
            assert_eq!(body.locomotion, Locomotion::Walk);
            let ratio = body.height / (MAMMAL * mass.cbrt());
            assert!(
                (0.75..1.25).contains(&ratio),
                "{mass} kg grazer {:.2} m at the hip",
                body.height
            );
            if mass == 75.0 {
                assert!(body.height >= 0.7, "{}", body.height);
            }
        }
    }
}
