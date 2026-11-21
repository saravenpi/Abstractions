//! Tests of where stones come to rest: never on ground steeper than their angle of repose, never on a
//! crest, a ridge or a peak, tilted to the ground they lie on, and each plain rock one of the hard
//! stones of its planet.

use std::sync::Arc;

use super::ctx::Ctx;
use super::perch::{Lie, repose, welcome};
use super::relief::Relief;
use super::tests_river::{river_chunks, rivers};
use super::tile::Tile;
use super::zones::{Zone, chunks, tiles};
use super::*;
use crate::universe::{PlanetRecord, Water};
use crate::world::globe::Globe;
use crate::world::rocks::tune::{CREST, HOLLOW};

/// What one stone of the scatter lies on.
struct Rest {
    /// The stone.
    plant: Plant,
    /// Slope of the drawn ground under it, rise over run, as the scatter reads it.
    slope: f32,
    /// The lie of the ground there.
    lie: Lie,
    /// The grade the stone is tilted to.
    lean: Vec2,
}

/// Returns the context a pass over the chunk of `tile` reads, over the `heights` of its mesh.
fn read<'a>(tile: &'a Tile, heights: &'a [Vec<f32>]) -> Ctx<'a> {
    Ctx::new(
        &tile.terrain,
        tile.coord,
        Some(Relief::new(tile.coord, heights)),
    )
}

/// Returns every resting stone among the big props of `tile`, with the ground it lies on.
fn rests(tile: &Tile) -> Vec<Rest> {
    let heights = chunk_heights(&tile.terrain, tile.coord, 1);
    let ctx = read(tile, &heights);
    let props = scatter_over(&tile.terrain, tile.coord, &heights).props;
    let stones = props.into_iter().filter(|p| repose(p.plant).is_some());
    stones
        .filter_map(|p| {
            let w = ctx.center + p.pos.xz();
            let at = ctx.look(w)?;
            Some(Rest {
                plant: p.plant,
                slope: at.slope,
                lie: ctx.lie(w),
                lean: p.lean,
            })
        })
        .collect()
}

/// Returns the steepest slope of the drawn ground of `tile`, rise over run, on a grid of 2 m.
fn steepest(tile: &Tile) -> f32 {
    let heights = chunk_heights(&tile.terrain, tile.coord, 1);
    let ctx = read(tile, &heights);
    let corner = chunk_center(tile.coord) - Vec2::splat(CHUNK_SIZE * 0.5);
    let grid =
        (0..32 * 32).map(|i| corner + Vec2::new((i % 32) as f32, (i / 32) as f32) * 2.0 + 1.0);
    grid.filter_map(|w| ctx.look(w))
        .map(|g| g.slope)
        .fold(0.0, f32::max)
}

/// Every stone lies on drawn ground no steeper than its angle of repose, on no spot higher over its
/// ring than the crest margin, and tilted to the ground under it: on two globes at face centres, edges
/// and corners, and on the highlands that send rivers down, which hold ground steeper than any repose.
#[test]
fn stones_rest_only_where_they_can() {
    let mut all: Vec<Tile> = tiles(15, 3).into_iter().map(|(_, t)| t).collect();
    all.extend(river_chunks(&rivers(), 6).into_iter().map(|(t, _)| t));
    let mut seen = 0;
    for rest in all.iter().flat_map(rests) {
        let limit = repose(rest.plant).unwrap_or(0.0).to_radians().tan();
        assert!(
            rest.slope <= limit + 1e-4,
            "{:?} on a slope of {}",
            rest.plant,
            rest.slope
        );
        assert!(
            rest.lie.crest < CREST,
            "{:?} on a crest {} m over its ring",
            rest.plant,
            rest.lie.crest
        );
        let off = (rest.lean - rest.lie.grade).length();
        assert!(off < 1e-5, "{:?} leans {off} off its ground", rest.plant);
        seen += 1;
    }
    let steep = all.iter().map(steepest).fold(0.0, f32::max);
    assert!(
        seen > 30 && steep > 0.8,
        "{seen} stones, steepest ground {steep}"
    );
}

/// A stone is welcome once on flat or evenly sloping ground, the most in a hollow as deep as the crest
/// margin, never on a crest of the margin, and never past its angle of repose: blocky basalt holds a
/// 35 degree slope that slick ice slides off. Every repose lies between 25 and 40 degrees.
#[test]
fn a_stone_is_welcome_in_hollows_and_never_on_crests() {
    let lie = |slope: f32, crest: f32| Lie {
        grade: Vec2::new(slope, 0.0),
        crest,
    };
    assert_eq!(welcome(Plant::Rock, &lie(0.0, 0.0)), 1.0);
    assert_eq!(welcome(Plant::Rock, &lie(0.3, 0.0)), 1.0);
    assert_eq!(welcome(Plant::Rock, &lie(0.0, -CREST)), HOLLOW);
    assert_eq!(welcome(Plant::Rock, &lie(0.0, CREST)), 0.0);
    assert_eq!(welcome(Plant::Rock, &lie(1.0, 0.0)), 0.0);
    assert!(welcome(Plant::Rock, &lie(0.7, 0.0)) > 0.0);
    assert_eq!(welcome(Plant::IceRock, &lie(0.7, 0.0)), 0.0);
    let stones = [
        Plant::Rock,
        Plant::IceRock,
        Plant::Sandstone,
        Plant::Flint,
        Plant::Obsidian,
        Plant::Boulder,
    ];
    for plant in stones {
        let degrees = repose(plant).expect("a stone has an angle of repose");
        assert!((25.0..=40.0).contains(&degrees), "{plant:?} at {degrees}");
    }
    assert_eq!(repose(Plant::Oak), None);
}

/// Returns every plant the chunks of `tiles` hold once they lie on `globe`.
fn kinds(globe: &Arc<Globe>, tiles: &[Tile]) -> Vec<Plant> {
    let on = tiles.iter().map(|t| Tile::new(globe, t.face, t.coord));
    on.flat_map(|t| t.scatter()).map(|(p, _)| p.plant).collect()
}

/// A plain rock of the tables is one of the hard stones of its planet: basalt, flint and obsidian all
/// lie on the home globe, and a world without liquid water has rocks but no flint.
#[test]
fn plain_rocks_are_the_stones_of_their_planet() {
    let home = Arc::new(Globe::new(5, 6000.0, 22.0));
    let land = chunks(&home, Zone::Centre, 10, 40);
    let found = kinds(&home, &land);
    for stone in [Plant::Rock, Plant::Flint, Plant::Obsidian] {
        assert!(found.contains(&stone), "no {stone:?} on the home globe");
    }
    let record = PlanetRecord {
        water: Water::None,
        ..PlanetRecord::TEMPERATE
    };
    let dry = Arc::new(Globe::new(5, 6000.0, 22.0).with_record(record));
    let found = kinds(&dry, &land);
    assert!(found.contains(&Plant::Rock) && !found.contains(&Plant::Flint));
}
