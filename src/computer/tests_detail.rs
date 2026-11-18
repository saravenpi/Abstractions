//! Computer tests: the readout of each class, when a record is known and the map marks.

use std::collections::HashSet;
use std::f64::consts::PI;

use bevy::prelude::*;

use super::chart::{GRID, mark};
use super::detail::{LINES, UNCHARTED, brief, readout, worlds};
use super::*;
use crate::universe::{PlanetClass, PlanetRecord, PlanetSpec, StarId, Universe, Water, Zone};

/// One hand-built record per class, in the order of `PlanetClass::ALL`, none the home record.
pub(super) fn records() -> [PlanetRecord; 6] {
    let (wet, dry, ice, gas) = (Water::Liquid, Water::None, Water::Ice, Water::Steam);
    let (hot, mild, cold) = (Zone::Hot, Zone::Habitable, Zone::Cold);
    let heat = [295.15, 287.15, 318.15, 153.15, 1473.15, 0.0];
    let air = [1.2, 1.0, 0.4, 0.2, 92.0, 0.0];
    let tilt = [0.2, 0.409, 0.1, 1.0, 0.0, PI];
    let water = [wet, wet, dry, ice, gas, dry];
    let ocean = [0.95, 0.6, 0.0, 0.0, 0.3, 0.0];
    let flux = [1.0, 1.0, 1.8, 0.04, 1500.0, 0.004];
    let zone = [mild, mild, mild, cold, hot, cold];
    std::array::from_fn(|i| PlanetRecord {
        class: PlanetClass::ALL[i],
        temperature: heat[i],
        pressure: air[i],
        tilt: tilt[i],
        water: water[i],
        ocean: ocean[i],
        flux: flux[i],
        zone: zone[i],
        ..PlanetRecord::TEMPERATE
    })
}

/// Returns a planet of 6 km with a day of 24 minutes and a year of 200 minutes carrying
/// `record`, and its row.
pub(super) fn world(record: PlanetRecord) -> (Line, PlanetSpec) {
    let universe = Universe::new(7);
    let home = universe.home();
    let planet = PlanetSpec {
        radius: 6000.0,
        gravity: crate::universe::HOME_GRAVITY,
        day_seconds: 1440.0,
        orbit_seconds: 12_000.0,
        record,
        ..universe.planet(home)
    };
    let line = Line {
        title: "Kodya".to_string(),
        detail: format!(
            "1.00 km - {} - radius 6.00 km - 1.00 g",
            brief(Some(&record))
        ),
        target: Target::Body(home),
        distance: 1000.0,
        visited: false,
        at: default(),
    };
    (line, planet)
}

/// The row of each class starts with its name and its temperature, in few words.
#[test]
fn every_class_has_a_short_brief() {
    let shown = records().map(|record| brief(Some(&record)));
    let expected = [
        "ocean - 22 C",
        "island - 14 C",
        "desert - 45 C",
        "ice - -120 C",
        "lava - 1200 C",
        "barren - -273 C",
    ];
    assert_eq!(shown, expected);
    assert_eq!(brief(None), UNCHARTED);
    assert_eq!(brief(Some(&PlanetRecord::TEMPERATE)), "island - 15 C");
    let classes: HashSet<PlanetClass> = records().iter().map(|record| record.class).collect();
    assert_eq!(classes.len(), PlanetClass::ALL.len());
}

/// The readout of each class says the whole record in plain words and units.
#[test]
fn the_readout_says_the_whole_record_of_each_class() {
    let lines = records().map(|record| {
        let (line, planet) = world(record);
        readout(&line, Some(&planet))[..4].join(" | ")
    });
    let expected = [
        "ocean - 22 C - tilt 11 deg | air like home, pressure 1.20 | \
         water liquid, oceans 95 % | light 1.00 x home, zone habitable",
        "island - 14 C - tilt 23 deg | air like home, pressure 1.00 | \
         water liquid, oceans 60 % | light 1.00 x home, zone habitable",
        "desert - 45 C - tilt 6 deg | air thin, pressure 0.40 | \
         water none | light 1.80 x home, zone habitable",
        "ice - -120 C - tilt 57 deg | air thin, pressure 0.20 | \
         water ice | light 0.04 x home, zone too cold",
        "lava - 1200 C - tilt 0 deg | air thick, pressure 92.0 | \
         water steam, molten rock 30 % | light 1500 x home, zone too hot",
        "barren - -273 C - tilt 180 deg | air none, pressure 0.00 | \
         water none | light 0.00 x home, zone too cold",
    ];
    assert_eq!(lines, expected);
    let (line, planet) = world(PlanetRecord::TEMPERATE);
    let home = readout(&line, Some(&planet));
    assert_eq!(home[4], "radius 6.00 km - gravity 1.00 g");
    assert_eq!(home[5], "day 24 min - year 3 h 20 min");
    assert_eq!(readout(&line, None), [line.facts()]);
}

/// Returns the rows of the system of `star` seen from far away, and the planets whose record
/// `atlas` knows.
fn rows(universe: &Universe, atlas: &Atlas, star: StarId) -> (Vec<Line>, Vec<Option<PlanetSpec>>) {
    let (ship, time) = (default(), 0.0);
    let view = View {
        universe,
        atlas,
        ship,
        time,
    };
    let lines = view.system(star);
    let known = worlds(universe, atlas, &lines);
    (lines, known)
}

/// The record of a planet is known once its system is visited and not before, like the
/// planet count of its star, and nothing but the visit is stored.
#[test]
fn a_record_is_known_once_the_system_is_visited() {
    let universe = Universe::new(7);
    let mut atlas = Atlas::new(&universe);
    let home = universe.home().star;
    let far = atlas.known_systems(&universe, home.galaxy);
    let far = far.iter().find(|star| star.id != home).unwrap().id;
    let (lines, known) = rows(&universe, &atlas, far);
    assert!(lines.len() > 1 && known.iter().all(Option::is_none));
    let hidden = |line: &Line| line.facts().starts_with("uncharted - radius ");
    assert!(lines[1..].iter().all(hidden));
    let saved = toml::to_string(&atlas).unwrap().len();
    atlas.visit(Target::Star(far));
    let (lines, known) = rows(&universe, &atlas, far);
    assert!(known[0].is_none() && known[1..].iter().all(Option::is_some));
    for (line, planet) in lines.iter().zip(&known).skip(1) {
        let planet = planet.as_ref().unwrap();
        assert_eq!(line.target, Target::Body(planet.id));
        assert!(line.facts().starts_with(&brief(Some(&planet.record))));
        assert_eq!(readout(line, Some(planet)).len(), LINES);
    }
    let grown = toml::to_string(&atlas).unwrap().len() - saved;
    assert!(grown < 200, "a visit stores one id, not {grown} bytes");
}

/// Every class has its own mark on the map, made of blocks that stay inside the grid and
/// never overlap.
#[test]
fn every_class_has_its_own_mark() {
    let cells = |class: PlanetClass| {
        let mut filled = Vec::new();
        for [x, y, w, h] in mark(class) {
            assert!(
                *w > 0 && *h > 0 && x + w <= GRID && y + h <= GRID,
                "{class:?}"
            );
            filled.extend((*x..x + w).flat_map(|i| (*y..y + h).map(move |j| (i, j))));
        }
        let unique: HashSet<(u8, u8)> = filled.iter().copied().collect();
        assert_eq!(
            unique.len(),
            filled.len(),
            "{class:?} has overlapping blocks"
        );
        filled.sort_unstable();
        filled
    };
    let marks: HashSet<Vec<(u8, u8)>> = PlanetClass::ALL.into_iter().map(cells).collect();
    assert_eq!(marks.len(), PlanetClass::ALL.len());
    assert_eq!(cells(PlanetClass::Island).len(), usize::from(GRID * GRID));
}
