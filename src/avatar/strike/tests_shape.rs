//! Geometry and proportion tests for procedural 3D tool shapes across every tool figure.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::shape::{Part, figure, parts};
use super::shape_mesh::{FormMesh, Meshes};
use super::show::{blocks, kit, tool};
use crate::craft::build::{Build, Figure, Form, Hands, Head, Shaft, Weight};
use crate::craft::matter::{Class, Material};

/// Returns a synthetic tool build configured for testing a specific figure.
fn make_build(form: Form, shaft_len: f32, head_mass: f32, weight_mass: Option<f32>) -> Build {
    let wood = Class::Wood.anchor();
    let stone = Material {
        density: 2600.0,
        ..wood
    };
    Build {
        grip: 0.12,
        shaft: Shaft {
            length: shaft_len,
            diameter: 0.032,
            material: wood,
        },
        head: Head {
            form,
            mass: head_mass,
            material: stone,
        },
        counterweight: weight_mass.map(|mass| Weight {
            mass,
            material: stone,
        }),
        binding: Some(Class::Hide.anchor()),
        hands: Hands::One,
    }
}

/// Returns the `(min, max)` axis-aligned bounds across `items`.
fn span(items: &[Part]) -> (Vec3, Vec3) {
    items.iter().fold((Vec3::MAX, Vec3::MIN), |(lo, hi), part| {
        (
            lo.min(part.at - part.size * 0.5),
            hi.max(part.at + part.size * 0.5),
        )
    })
}

/// Procedural unit meshes are non-empty triangle lists with outward normals aligned to winding.
#[test]
fn procedural_unit_meshes_have_outward_normals() {
    let mut assets = Assets::<Mesh>::default();
    let kit_meshes = Meshes::new(&mut assets);
    for kind in [
        FormMesh::Wedge,
        FormMesh::AxeBit,
        FormMesh::Blade,
        FormMesh::Point,
    ] {
        let mesh = assets.get(kit_meshes.get(kind)).expect("mesh loaded");
        let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("{kind:?} missing positions");
        };
        let Some(VertexAttributeValues::Float32x3(nor)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("{kind:?} missing normals");
        };
        assert!(!pos.is_empty() && pos.len() == nor.len());
        let indices = mesh.indices().expect("indexed").iter().collect::<Vec<_>>();
        for tri in indices.as_chunks::<3>().0 {
            let (a, b, c) = (
                Vec3::from(pos[tri[0]]),
                Vec3::from(pos[tri[1]]),
                Vec3::from(pos[tri[2]]),
            );
            let face_normal = (b - a).cross(c - a);
            let stored = Vec3::from(nor[tri[0]]);
            assert!(face_normal.length() > 1e-6);
            assert!(stored.length() > 0.99);
            assert!(face_normal.dot(stored) > 0.0, "{kind:?} flipped winding");
        }
    }
}

/// Axes and hammers extend horizontally off the shaft with distinct flared and cross-head shapes.
#[test]
fn axe_and_hammer_extend_across_shaft() {
    let axe_build = make_build(Form::Edge, 0.55, 1.4, None);
    assert_eq!(figure(&axe_build), Figure::Axe);
    let axe_parts = parts(&axe_build, 0.0);
    assert!(axe_parts.iter().any(|p| p.mesh == FormMesh::AxeBit));
    let (axe_lo, axe_hi) = span(&axe_parts);
    assert!(axe_hi.x > axe_build.shaft.diameter * 3.0);
    assert!(axe_hi.x > -axe_lo.x);

    let hammer_build = make_build(Form::Mass, 0.50, 1.6, None);
    assert_eq!(figure(&hammer_build), Figure::Hammer);
    let hammer_parts = parts(&hammer_build, 0.0);
    assert!(hammer_parts.iter().any(|p| p.mesh == FormMesh::Wedge));
    let head_only: Vec<_> = hammer_parts
        .iter()
        .copied()
        .filter(|p| p.paint == 1)
        .collect();
    let (h_lo, h_hi) = span(&head_only);
    assert!(h_hi.x > 0.05 && h_lo.x < -0.05);
    assert!(h_hi.x - h_lo.x > (h_hi.y - h_lo.y) * 1.4);
}

/// Swords keep only a grip below a crossguard and form a full-length blade and point.
#[test]
fn blade_and_knife_replace_long_wooden_shafts_with_blades() {
    let sword_build = make_build(Form::Edge, 0.65, 0.45, Some(0.15));
    assert_eq!(figure(&sword_build), Figure::Blade);
    let sword_parts = parts(&sword_build, 0.0);
    let grip = sword_parts
        .iter()
        .find(|p| p.paint == 0)
        .expect("sword grip");
    assert!(grip.at.y + grip.size.y * 0.5 < 0.14);
    assert!(sword_parts.iter().any(|p| p.mesh == FormMesh::Blade));
    assert!(sword_parts.iter().any(|p| p.mesh == FormMesh::Point));
    let (_, sword_hi) = span(&sword_parts);
    assert!((sword_hi.y - sword_build.reach()).abs() < 1e-4);

    let knife_build = make_build(Form::Edge, 0.25, 0.18, None);
    assert_eq!(figure(&knife_build), Figure::Knife);
    let knife_parts = parts(&knife_build, 0.0);
    assert!(knife_parts.iter().any(|p| p.mesh == FormMesh::Wedge));
    assert!(knife_parts.iter().any(|p| p.mesh == FormMesh::Point));
    let (_, knife_hi) = span(&knife_parts);
    assert!((knife_hi.y - knife_build.reach()).abs() < 1e-4);
}

/// Spears, javelins, clubs and the spawned `tool` entity assemble cleanly across all figures.
#[test]
fn spear_javelin_club_and_spawner_match_reach() {
    let builds = [
        (make_build(Form::Point, 1.1, 0.25, None), Figure::Spear),
        (
            make_build(Form::Point, 1.0, 0.22, Some(0.15)),
            Figure::Javelin,
        ),
        (make_build(Form::Mass, 0.70, 0.40, None), Figure::Club),
    ];
    let mut app = App::new();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<StandardMaterial>::default();
    let built_kit = kit(&mut meshes, &mut materials);
    for (build, expected) in builds {
        assert_eq!(figure(&build), expected);
        let boxes = blocks(&build, 0.0);
        let max_y = boxes
            .iter()
            .map(|(at, sz, _)| at.y + sz.y * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y - build.reach()).abs() < 1e-3, "{expected:?}");
        let entity = tool(
            &mut app.world_mut().commands(),
            &built_kit,
            &build,
            None,
            0.0,
        );
        assert!(entity != Entity::PLACEHOLDER);
    }
}
