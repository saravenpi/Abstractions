//! Tests of scatter under the climate: nothing grows above the snow line, and plants thin out as the
//! air leaves the warmth of their biome.

use std::sync::Arc;

use super::spacing::stone;
use super::tile::Tile;
use super::zones::{Zone, chunks};
use super::*;
use crate::climate::SNOW_LINE as FROST;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::biome::Biome;
use crate::world::globe::Globe;
use crate::world::lands::thrive::{TREE_LINE, climate_at};

/// Returns a 6 km globe of seed 5 with the home record changed into `class`, `ocean` and
/// `temperature` kelvin, under `pressure` times the air of home.
fn globe(class: PlanetClass, ocean: f64, temperature: f64, pressure: f64) -> Arc<Globe> {
    let record = PlanetRecord {
        class,
        ocean,
        temperature,
        pressure,
        ..PlanetRecord::TEMPERATE
    };
    Arc::new(Globe::new(5, 6000.0, 22.0).with_record(record))
}

/// Returns the land props of up to `want` mostly dry chunks around the face centres of `globe`, with
/// the view of their face and their chart position.
fn land(globe: &Arc<Globe>, want: usize) -> Vec<(Terrain, Vec2, Placement)> {
    let props = |t: Tile| {
        let s = scatter_chunk(&t.terrain, t.coord);
        let all = s.props.into_iter().chain(s.grass).filter(|p| p.pos.y > 0.0);
        let at = |p: Placement| (t.terrain.clone(), chunk_center(t.coord) + p.pos.xz(), p);
        all.map(at).collect::<Vec<_>>()
    };
    chunks(globe, Zone::Centre, 20, want)
        .into_iter()
        .flat_map(props)
        .collect()
}

/// On a world much colder than home every land prop under the snow line of the climate is a stone,
/// except the pines of the taiga, which stand down to its tree line, and the stones still lie
/// there. The same planet gives the same props twice.
#[test]
fn nothing_grows_above_the_snow_line() {
    let cold = globe(
        PlanetClass::Island,
        PlanetRecord::BALANCED_OCEAN,
        250.0,
        1.0,
    );
    let found = land(&cold, 8);
    assert!(
        found.iter().any(|(_, _, p)| stone(p.plant)),
        "no stones on the cold land"
    );
    for (terrain, w, p) in &found {
        let air = climate_at(terrain, *w).expect("a globe has a climate");
        let line = match terrain.biome(w.x, w.y) {
            Biome::Taiga => TREE_LINE,
            _ => FROST,
        };
        let ok = stone(p.plant) || air.temperature >= line - 0.5;
        assert!(ok, "{:?} grows in {} K", p.plant, air.temperature);
    }
    let again = land(&cold, 8).into_iter().map(|(_, _, p)| p);
    assert!(again.eq(found.iter().map(|(_, _, p)| *p)));
}

/// Returns how many land props of `globe` are plants rather than stones.
fn plants(globe: &Arc<Globe>) -> usize {
    land(globe, 16)
        .iter()
        .filter(|(_, _, p)| !stone(p.plant))
        .count()
}

/// A desert world, whose biome never moves, under air thick enough to even out its latitudes,
/// grows thick where its air is the heart of a desert and far thinner where it is much hotter.
#[test]
fn plants_thin_out_away_from_the_heart_of_their_biome() {
    let heart = plants(&globe(PlanetClass::Desert, 0.0, 305.0, 400.0));
    let scorched = plants(&globe(PlanetClass::Desert, 0.0, 340.0, 400.0));
    assert!(heart > 30, "only {heart} plants in the heart of the desert");
    assert!(
        (scorched as f32) < heart as f32 * 0.75,
        "{scorched} plants in scorched air against {heart}"
    );
}
