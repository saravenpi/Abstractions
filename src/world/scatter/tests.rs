//! Tests of the plant tables and of the placement budgets.

use super::rules::{check_pond, check_sea, underwater};
use super::*;
use crate::world::propmesh::{BIG, TREES};
use crate::world::{Biome, CHUNK_SIZE};
use Plant::*;

/// Sample chunks around the origin of seed 5.
fn chunks() -> impl Iterator<Item = IVec2> {
    (-5..5).flat_map(|x| (-5..5).map(move |y| IVec2::new(x, y)))
}

/// The same seed and chunk always give the same props, and islands get some.
#[test]
fn scatter_is_deterministic() {
    let t = Terrain::new(5);
    let spawn = t.find_spawn();
    let coord = IVec2::new(
        (spawn.x / CHUNK_SIZE).floor() as i32,
        (spawn.z / CHUNK_SIZE).floor() as i32,
    );
    let a = scatter_chunk(&t, coord);
    assert_eq!(a, scatter_chunk(&Terrain::new(5), coord));
    assert!(!a.grass.is_empty());
    assert!(a.props.len() <= BIG_CAP && a.grass.len() <= SMALL_CAP);
}

/// A chunk whose floor is everywhere deeper than the limit holds nothing.
#[test]
fn deep_sea_is_empty() {
    let t = Terrain::new(5);
    let floor = |c: IVec2| {
        let o = chunk_center(c) - Vec2::splat(CHUNK_SIZE * 0.5);
        (0..25)
            .map(|i| o + Vec2::new((i % 5) as f32, (i / 5) as f32) * CHUNK_SIZE / 4.0)
            .map(|p| t.height(p.x, p.y))
            .fold(f32::MIN, f32::max)
    };
    let deep = (0..400)
        .map(|x| IVec2::new(x * 7, 1000))
        .find(|c| floor(*c) < -DEEP_LIMIT)
        .expect("a deep chunk exists far from the islands");
    assert_eq!(scatter_chunk(&t, deep), Scatter::default());
}

/// Land trees stay on land, nothing underwater grows on land and each biome keeps its own plants.
#[test]
fn plants_follow_biome_and_surface() {
    let dry = [Surface::Sand, Surface::Grass, Surface::Rock, Surface::Snow];
    let has = |b: Biome, s: Surface, p: Plant| plants_for(b, s).iter().any(|e| e.0 == p);
    let anywhere = |b: Biome, p: Plant| dry.iter().any(|&s| has(b, s, p));
    for b in Biome::ALL {
        [Surface::Water, Surface::Shallows, Surface::Reef]
            .into_iter()
            .for_each(|s| check_sea(b, s));
        check_pond(b);
        for s in dry {
            assert!(
                plants_for(b, s)
                    .iter()
                    .all(|e| !underwater(e.0) && !e.0.pond())
            );
        }
        assert_eq!(anywhere(b, Saguaro), b == Biome::Desert);
        assert_eq!(
            anywhere(b, Willow),
            !matches!(b, Biome::Desert | Biome::Taiga)
        );
        let trees = |s: Surface| plants_for(b, s).iter().any(|e| (e.0 as usize) < TREES);
        assert!(!trees(Surface::Rock) && trees(Surface::Snow) == (b == Biome::Taiga));
    }
    assert!(has(Biome::Taiga, Surface::Snow, SnowPine));
    assert!(anywhere(Biome::Tropical, Palm) && anywhere(Biome::Cherry, Cherry));
}

/// Budgets hold, kelp stays under the surface and nothing submerged pokes out of the sea.
#[test]
fn budgets_and_water_line_hold() {
    let t = Terrain::new(5);
    for coord in chunks() {
        let s = scatter_chunk(&t, coord);
        assert!(s.props.len() <= BIG_CAP && s.props.iter().all(|p| (p.plant as usize) < BIG));
        let tris: usize = s.grass.iter().map(small_tris).sum();
        assert!(
            tris < TRI_BUDGET + 150,
            "chunk {coord} has {tris} triangles"
        );
        for p in s
            .props
            .iter()
            .chain(&s.grass)
            .filter(|p| p.pos.y < SEA_LEVEL)
        {
            let top = if p.plant == Kelp {
                p.scale + 1.0
            } else {
                p.plant.draft() * p.scale
            };
            assert!(
                p.pos.y + top <= 1e-3,
                "{:?} sticks out at {}",
                p.plant,
                p.pos.y
            );
        }
    }
}
