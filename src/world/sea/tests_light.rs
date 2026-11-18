//! Tests of the light of the sea: the shader holds the day edges of the Rust side, and the water
//! material takes the light of the planet under the camera.

use bevy::prelude::*;

use super::Sea;
use super::follow::light;
use super::material::WaterMaterial;
use super::uniform::{DAY_EDGE, SUN_UP_EDGE, WaterUniform};
use crate::render::{PlaceLight, PlaceUniform};

/// The water shader.
const SHADER: &str = include_str!("water.wesl");

/// Returns the two numbers of the constant `name` of the water shader.
fn shader_edge(name: &str) -> (f32, f32) {
    let start = format!("const {name}:");
    let line = SHADER.lines().find(|line| line.starts_with(&start));
    let value = line.and_then(|line| line.split('=').nth(1)).unwrap_or("");
    let bare = value.replace("vec2<f32>", "");
    let numbers = bare.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'));
    let mut found = numbers.filter_map(|number| number.parse().ok());
    (
        found.next().unwrap_or(f32::NAN),
        found.next().unwrap_or(f32::NAN),
    )
}

/// The shader moves the day and the glitter of far water by the same edges the Rust side gives
/// the water at the camera, and it reads the light of the planet and lights the far sea with it.
#[test]
fn the_shader_holds_the_day_edges() {
    assert_eq!(shader_edge("DAY_EDGE"), DAY_EDGE);
    assert_eq!(shader_edge("SUN_UP_EDGE"), SUN_UP_EDGE);
    assert!(SHADER.contains("@binding(3) var<uniform> place: Place;"));
    assert!(SHADER.contains("relit(input, apply_pbr_lighting(input), place)"));
}

/// The water material takes the light of the planet when it changes and keeps its other values.
#[test]
fn the_sea_takes_the_light_of_the_planet() {
    let mut app = App::new();
    let mut materials = Assets::<WaterMaterial>::default();
    let water = WaterUniform {
        params: Vec4::new(3.0, 0.5, 120.0, 420.0),
        ..default()
    };
    let handle = materials.add(WaterMaterial {
        water,
        floor: Handle::default(),
        place: default(),
    });
    app.insert_resource(materials)
        .init_resource::<PlaceLight>()
        .add_systems(Update, light);
    app.world_mut().spawn((Sea, MeshMaterial3d(handle.clone())));
    let lit = PlaceUniform {
        star: Vec4::new(0.0, 1.0, 0.0, 1.0),
        air: Vec4::new(4000.0, 60.0, 1.0, 0.0),
        ..default()
    };
    app.world_mut().resource_mut::<PlaceLight>().0 = lit;
    app.update();
    let materials = app.world().resource::<Assets<WaterMaterial>>();
    let sea = materials.get(&handle).expect("the water material exists");
    assert_eq!((sea.place, sea.water.params), (lit, water.params));
}
