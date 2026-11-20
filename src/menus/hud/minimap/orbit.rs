//! The space view as plain data: the chart of the system around the ship, drawn with the
//! projection of the board computer, and the lens that keeps the nearest body and the
//! destination in sight.

use bevy::math::{IVec2, Vec2};

use crate::computer::{Dot, Lens, Line, MOON_ZOOM, SIZE, Tab, Target, View, chart};
use crate::universe::{PlanetSpec, StarId};

/// Share of the half width of the map the ship, the nearest body and the destination span.
const FILLED: f32 = 0.7;

/// Closest zoom of the minimap, which keeps the neighbours in sight while the ship orbits.
pub const CLOSEST: f32 = MOON_ZOOM;

/// Cells per chart unit of the grid a mark has to leave before the chart is drawn again: a
/// cell is a sixth of a map pixel on the whole chart, a pixel a few zooms in.
const GRID: f32 = SIZE * 3.0;

/// The chart of one system, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// The star, then its planets outward.
    pub lines: Vec<Line>,
    /// For each row, the planet whose class is known.
    pub worlds: Vec<Option<PlanetSpec>>,
    /// Every dot of the chart with its place.
    pub dots: Vec<(Dot, Vec2)>,
    /// The row of the destination, when it is in this system.
    pub chosen: Option<usize>,
    /// How the chart is looked at.
    pub lens: Lens,
}

impl Scene {
    /// Returns the place on the chart of `wanted`, when the chart has it.
    pub fn place(&self, wanted: Dot) -> Option<Vec2> {
        let found = self.dots.iter().find(|(dot, _)| *dot == wanted);
        found.map(|(_, at)| *at)
    }
}

/// Returns the lens that shows the ship, the body nearest to it and the row `chosen` with room
/// around them, never closer than [`CLOSEST`].
pub fn lens(dots: &[(Dot, Vec2)], chosen: Option<usize>) -> Lens {
    let place = |wanted: Dot| {
        dots.iter()
            .find(|(dot, _)| *dot == wanted)
            .map(|(_, at)| *at)
    };
    let ship = place(Dot::Ship).unwrap_or_default();
    let rows = dots.iter().filter(|(dot, _)| matches!(dot, Dot::Row(_)));
    let gap = |at: &Vec2| at.distance(ship);
    let nearest = rows
        .map(|(_, at)| *at)
        .min_by(|a, b| gap(a).total_cmp(&gap(b)));
    let seen = [nearest, chosen.and_then(|row| place(Dot::Row(row)))];
    let reach = |(lo, hi): (Vec2, Vec2), at: &Vec2| (lo.min(*at), hi.max(*at));
    let (lo, hi) = seen.iter().flatten().fold((ship, ship), reach);
    let half = ((hi - lo).max_element() * 0.5).max(FILLED / CLOSEST);
    Lens {
        zoom: FILLED / half,
        centre: (lo + hi) * 0.5,
    }
    .clamped()
}

/// Returns the chart of the system of `star` for what `view` knows, with the destination
/// `chosen`. The class of a planet shows once its system was visited.
pub fn scene(view: &View, star: StarId, chosen: Option<Target>) -> Scene {
    let lines = view.system(star);
    let dots = chart(Tab::System, &lines, view.ship);
    let chosen = lines.iter().position(|line| Some(line.target) == chosen);
    let world = |line: &Line| match line.target {
        Target::Body(id) if view.atlas.visited(Target::Star(id.star)) => {
            Some(view.universe.planet(id))
        }
        _ => None,
    };
    Scene {
        worlds: lines.iter().map(world).collect(),
        lens: lens(&dots, chosen),
        lines,
        dots,
        chosen,
    }
}

/// Returns what the drawn chart depends on: the destination, whether the moons part from their
/// planets, and every mark of the system on a fine grid. The ship is left out, it moves on its
/// own. The chart is drawn again only when this changes.
pub fn signature(scene: &Scene) -> Vec<IVec2> {
    let fixed = |dot: &Dot| !matches!(dot, Dot::Orbit | Dot::Ship);
    let marks = scene.dots.iter().filter(|(dot, _)| fixed(dot));
    let cells = marks.map(|(_, at)| (*at * GRID).round().as_ivec2());
    let apart = i32::from(scene.lens.zoom >= MOON_ZOOM);
    let chosen = scene.chosen.map_or(-1, |row| row as i32);
    std::iter::once(IVec2::new(2 + apart, chosen))
        .chain(cells)
        .collect()
}
