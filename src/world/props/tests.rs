//! Checks that ground cover and prop colliders placed through a rig keep their true size on the globe.

use avian3d::parry::math::Pose;

use super::*;
use crate::world::Terrain;
use crate::world::propmesh::Plant;
use crate::world::scatter::tile::extent;
use crate::world::scatter::zones::{Tile, trio};
use crate::world::scatter::{Scatter, scatter_chunk};
use crate::world::solids::tests::triangles;

/// Returns a small plant near the rim of its chunk at height `y`, of scale or kelp height `scale`.
fn sample(plant: Plant, y: f32, scale: f32) -> Placement {
    Placement {
        pos: Vec3::new(-27.0, y, 26.0),
        yaw: 2.3,
        scale,
        plant,
        variant: 0,
        lean: Vec2::ZERO,
    }
}

/// Returns the translation and rotation of a collider part.
fn pose(part: &Pose) -> (Vec3, Quat) {
    let (t, r) = (part.translation, part.rotation);
    (
        Vec3::new(t.x, t.y, t.z),
        Quat::from_xyzw(r.x, r.y, r.z, r.w),
    )
}

/// Asserts that small plant `p` built through the rig of `tile` starts, ends and reaches as far as the flat one
/// within 1 mm, and that it ends as high above the sea level sphere as the flat one above sea level.
fn check_shape(tile: &Tile, p: Placement) {
    let (embed, turn) = (|p| tile.point(p), |p| tile.turn(p));
    let scatter = Scatter {
        props: Vec::new(),
        grass: vec![p],
    };
    let flat = extent(&build_grass(&scatter), p.pos, Vec3::Y);
    let rig = Rig {
        embed: &embed,
        turn: &turn,
    };
    let rigid = build_grass_on(&scatter, &rig);
    let (origin, up) = (tile.point(p.pos), tile.turn(p.pos) * Vec3::Y);
    let size = extent(&rigid, origin, up);
    for k in 0..3 {
        let off = (flat[k] - size[k]).abs();
        assert!(
            off < 1e-3,
            "{:?} differs by {off} m at {}",
            p.plant,
            tile.coord
        );
    }
    let radius = |v: Vec3| (tile.frame.origin + tile.frame.rot * v.as_dvec3()).length();
    let points = triangles(&rigid).into_iter().flatten();
    let top = points.map(radius).fold(0.0, f64::max) - tile.terrain.globe().radius();
    let off = (top as f32 - (p.pos.y + flat[1])).abs();
    assert!(off < 1e-3, "{:?} ends {top} m above the sea", p.plant);
}

/// Grass, kelp, a lily and a coral built through a rig keep their true size at a face centre, at an edge
/// midpoint and next to a corner. The kelp ends its true height above the floor, below the sea level sphere,
/// and the lily pad floats on that sphere.
#[test]
fn cover_keeps_its_shape_on_the_globe() {
    let plants = [
        sample(Plant::Grass, 7.0, 1.1),
        sample(Plant::Kelp, -9.0, 6.0),
        sample(Plant::Lily, -0.8, 1.0),
        sample(Plant::BrainCoral, -3.0, 1.2),
    ];
    for tile in trio() {
        for p in plants {
            check_shape(&tile, p);
        }
    }
}

/// The collider of a tree placed through a rig keeps its true radius and height, its middle lies half its height
/// up the local up from the origin of the tree, and its axis is that up, which `propmesh` checks is the true one.
#[test]
fn solids_stand_on_the_local_up() {
    let tree = sample(Plant::Oak, 14.0, 1.2);
    let [radius, height] = BODIES[Plant::Oak as usize].map(|v| v * tree.scale);
    for tile in trio() {
        let (embed, turn) = (|p| tile.point(p), |p| tile.turn(p));
        let rig = Rig {
            embed: &embed,
            turn: &turn,
        };
        let bodies = trunk_collider_on(&[tree], &rig).expect("an oak is solid");
        let parts = bodies
            .shape()
            .as_compound()
            .expect("bodies are a compound")
            .shapes();
        let (at, rot) = pose(&parts[0].0);
        let cylinder = parts[0].1.as_cylinder().expect("a trunk is a cylinder");
        assert!((cylinder.radius - radius).abs() < 1e-6);
        assert!((cylinder.half_height * 2.0 - height).abs() < 1e-6);
        let (origin, up) = (tile.point(tree.pos), tile.turn(tree.pos) * Vec3::Y);
        let off = at.distance(origin + up * height * 0.5);
        let lean = (rot * Vec3::Y).distance(up);
        assert!(
            off < 1e-3 && lean < 1e-5,
            "trunk {off} m off, leaning {lean}"
        );
    }
}

/// Returns the cylinders built for the solid props of a chunk in chunk space, upright for plants and
/// tilted to the ground for stones.
fn legacy(props: &[Placement]) -> Vec<(Vec3, Quat, Collider)> {
    let solid = props.iter().filter(|p| BODIES[p.plant as usize][0] > 0.0);
    solid
        .map(|p| {
            let [radius, height] = BODIES[p.plant as usize].map(|v| v * p.scale);
            let tilt = if p.lean == Vec2::ZERO {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_arc(Vec3::Y, Vec3::new(-p.lean.x, 1.0, -p.lean.y).normalize())
            };
            let rot = tilt * Quat::from_rotation_y(p.yaw);
            let at = p.pos + rot * Vec3::Y * (height * 0.5);
            (at, rot, Collider::cylinder(radius, height))
        })
        .collect()
}

/// On the legacy plane the colliders are bit for bit the upright cylinders v0.5 built, while
/// `trunk_collider_on` keeps only non-stone cylinders and omits stones handled by [`stone_collider_on`].
#[test]
fn plain_solids_are_the_legacy_ones() {
    let t = Terrain::new(5);
    let parts = |c: &Collider| c.shape().as_compound().map(|c| c.shapes().to_vec());
    let mut seen = 0;
    for coord in (-4..4).flat_map(|x| (-4..4).map(move |y| IVec2::new(x, y))) {
        let props = scatter_chunk(&t, coord).props;
        let want = legacy(&props);
        let Some(bodies) = body_collider(&props) else {
            assert!(want.is_empty());
            assert!(trunk_collider_on(&props, &Rig::PLAIN).is_none());
            continue;
        };
        let got = parts(&bodies).unwrap_or_default();
        let want = parts(&Collider::compound(want)).unwrap_or_default();
        assert_eq!(got.len(), want.len());
        for (a, b) in got.iter().zip(&want) {
            assert_eq!(pose(&a.0), pose(&b.0));
            seen += 1;
        }
        let non_stones = props
            .iter()
            .filter(|p| !is_stone(p.plant) && BODIES[p.plant as usize][0] > 0.0)
            .count();
        let trunk_count = trunk_collider_on(&props, &Rig::PLAIN)
            .and_then(|c| parts(&c))
            .map_or(0, |s| s.len());
        assert_eq!(trunk_count, non_stones);
    }
    assert!(seen > 100, "only {seen} colliders compared");
}

/// Big scaled mineral outcrops use exact mesh colliders so rays in empty air near narrow spires do
/// not hit an invisible cylinder wall while rays aimed at actual crystal faces hit cleanly.
#[test]
fn big_minerals_collide_strictly_on_their_mesh() {
    use avian3d::parry::query::Ray;

    for plant in [Plant::Quartz, Plant::Obsidian] {
        let scale = 8.0;
        let placed = Placement {
            pos: Vec3::ZERO,
            yaw: 0.0,
            scale,
            plant,
            variant: 0,
            lean: Vec2::ZERO,
        };
        let collider = stone_collider_on(&[placed], &Rig::PLAIN).expect("mineral has trimesh");
        assert!(collider.shape().as_trimesh().is_some());
        let empty_air = Ray::new([3.2, 5.2, -10.0].into(), [0.0, 0.0, 1.0].into());
        assert!(
            collider
                .shape()
                .cast_local_ray(&empty_air, 20.0, true)
                .is_none(),
            "{plant:?} hit empty air outside its crystal spires"
        );
        let core = Ray::new([0.0, 0.8, -10.0].into(), [0.0, 0.0, 1.0].into());
        assert!(
            collider.shape().cast_local_ray(&core, 20.0, true).is_some(),
            "{plant:?} missed its actual mineral mesh"
        );
    }
}
