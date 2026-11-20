//! Building the zoo from the life of a seed, and spawning it.

use std::path::PathBuf;

use abstractions::fauna::mesh::part_mesh;
use abstractions::fauna::motion::Motion;
use abstractions::life::body::{Body, Locomotion};
use abstractions::life::{self, Guild, Life, Species};
use abstractions::universe::Universe;
use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use super::*;

/// Returns the life of the home planet of `seed` and its gravity.
pub fn home_life(seed: u64) -> (Option<Life>, f32) {
    let universe = Universe::new(seed);
    let id = universe.home();
    let planet = universe.planet(id);
    let star = universe.system(id.star).star;
    let gravity = life::World::of(&planet, &star).gravity;
    (life::of(&planet, &star), gravity)
}

/// Returns whether `s` is an animal with a body to draw.
pub fn animal(s: &Species) -> bool {
    s.role.guild != Guild::Producer && !s.body.parts.is_empty()
}

/// Prints the home life of seeds 0 to `count`.
pub fn find(count: u64) {
    for seed in 0..count {
        let (life, gravity) = home_life(seed);
        let Some(life) = life else {
            println!("seed {seed}: dead, g {gravity:.2}");
            continue;
        };
        let animals: Vec<&Species> = life.species.iter().filter(|s| animal(s)).collect();
        let walkers = animals
            .iter()
            .filter(|s| s.body.locomotion == Locomotion::Walk)
            .count();
        println!(
            "seed {seed}: {:?}, {} animals, {walkers} walkers, g {gravity:.2}",
            life.rung,
            animals.len()
        );
    }
}

/// Returns the radius of the circle `body` walks.
pub fn circle(body: &Body) -> f32 {
    (1.5 * body.length).max(1.0)
}

/// Returns the zoo of `seed`, with no part entities yet: its animals from small to large in rows
/// of `ROW`, the line of them wrapped so the camera can see them all.
pub fn zoo(seed: u64, shots: Option<PathBuf>) -> (Zoo, Vec<Species>) {
    let (life, gravity) = home_life(seed);
    let title = match &life {
        Some(l) => format!("seed {seed}, {:?} life, g {gravity:.2}", l.rung),
        None => format!("seed {seed}, dead planet"),
    };
    let mut species: Vec<Species> = life
        .map(|l| l.species.into_iter().filter(animal).collect())
        .unwrap_or_default();
    species.sort_by(|a, b| a.body.length.total_cmp(&b.body.length));
    let mut beasts = Vec::new();
    let mut z = 0.0;
    for (row, kinds) in species.chunks(ROW).enumerate() {
        let gap = kinds.iter().map(|s| circle(&s.body)).fold(1.0, f32::max) + 0.5;
        z -= gap;
        let mut x = 0.0;
        for (i, s) in kinds.iter().enumerate() {
            let radius = circle(&s.body);
            x += radius + 0.5;
            let centre = Vec3::new(x, 0.0, z);
            x += radius + 0.5;
            let lift = if s.body.locomotion == Locomotion::Walk {
                0.0
            } else {
                1.0
            };
            let start = centre + Vec3::new(radius, lift, 0.0);
            let seed = (row * ROW + i) as u64 + 1;
            beasts.push(Beast {
                body: s.body.clone(),
                motion: Motion::new(&s.body, start, Vec3::NEG_Z, seed),
                parts: Vec::new(),
                centre,
                radius,
            });
        }
        z -= gap;
    }
    let zoo = Zoo {
        beasts,
        gravity,
        time: 0.0,
        shots,
        done: 0,
        title,
    };
    (zoo, species)
}

/// Spawns the ground, the lights, the camera, the label and one entity per drawn part.
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut zoo: ResMut<Zoo>,
    species: Res<Kinds>,
) {
    let skin = materials.add(StandardMaterial {
        perceptual_roughness: 0.8,
        ..default()
    });
    let far = |f: fn(&Beast) -> f32| zoo.beasts.iter().map(f).fold(1.0, f32::max);
    let (span, depth) = (
        far(|b| b.centre.x + b.radius),
        far(|b| b.radius - b.centre.z),
    );
    let size = span.max(depth) + 200.0;
    let centre = Vec3::new(0.5 * span, 0.0, -0.5 * depth);
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
        MeshMaterial3d(materials.add(Color::srgb(0.35, 0.4, 0.33))),
        Transform::from_translation(centre),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 9000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 400.0,
        ..default()
    });
    let orbit = Orbit {
        yaw: 0.0,
        pitch: 0.6,
        distance: (0.6 * span.max(depth)).max(6.0),
        target: centre + Vec3::Y * 0.5,
    };
    commands.spawn((Camera3d::default(), orbit, Transform::default()));
    commands.spawn((
        Text::new(zoo.title.clone()),
        GaitLabel,
        Node {
            position_type: PositionType::Absolute,
            top: px(12.0),
            left: px(12.0),
            ..default()
        },
    ));
    for (beast, s) in zoo.beasts.iter_mut().zip(&species.0) {
        beast.parts = s
            .body
            .parts
            .iter()
            .map(|part| {
                let tissue = s.tissues.get(usize::from(part.tissue))?;
                let mesh = meshes.add(part_mesh(part, tissue));
                Some(
                    commands
                        .spawn((Mesh3d(mesh), MeshMaterial3d(skin.clone())))
                        .id(),
                )
            })
            .collect();
    }
}
