//! Deterministic procedural cave network planner keyed by planet seed and 320 m surface cells.

use bevy::prelude::*;

use crate::universe::PlanetClass;
use crate::universe::hash::{Rng, child};
use crate::world::globe::Chart;
use crate::world::{CHUNK_SIZE, SEA_LEVEL, Terrain};

/// Width of one cave spatial cell in chart metres (divides `BLOCK = 3840 m` and `CHUNK_SIZE = 64 m`).
pub const CAVE_CELL: f32 = 320.0;

/// Minimum surface height above sea level for a cave entrance.
const MIN_SURFACE_Y: f32 = 8.0;

/// Share of dry land cells that spawn a cave entrance.
const SPAWN_CHANCE: f64 = 0.40;

/// Tangent of the maximum walkable tunnel slope (25 degrees, strictly under 26 degrees).
const MAX_SLOPE_TAN: f32 = 0.4663;

/// Minimum clearance in metres between cave ceiling and surface past the entrance collar.
const CEILING_CLEARANCE: f32 = 2.6;

/// Salt mixed into planet seed for cave generation.
const CAVE_SALT: u64 = 0xCA7E_5EED_91A3_4B21;

/// Subterranean theme assigned to a cave from its planet class and seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaveBiome {
    /// Hanging glowing leaf vines, ceiling moss mats, and cyan/emerald glow.
    BioluminescentGrotto,
    /// Giant glowing mineral crystal clusters, pale refracted light, and sharp stalactites.
    CrystalSpire,
    /// Smooth root-carved caverns, giant glowing fungal caps, and amber/teal bioluminescence.
    MycelialDepths,
}

/// Stable identifier of one procedural cave on a planet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CaveId {
    /// Cube face index, or 0 on a flat chart.
    pub face: u8,
    /// Cell X index counted from the middle of the chart.
    pub cell_x: i32,
    /// Cell Z index counted from the middle of the chart.
    pub cell_z: i32,
}

/// Surface entrance portal metadata for fast hole carving and streaming checks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CavePortal {
    /// Stable cave identifier.
    pub id: CaveId,
    /// Entrance center in the caller's chart coordinates.
    pub entry_xz: Vec2,
    /// Ground height at the entrance center in metres above sea level.
    pub entry_y: f32,
    /// Entrance opening radius in metres (`3.6..5.2`).
    pub entry_radius: f32,
    /// Visual and ecological cave theme.
    pub biome: CaveBiome,
    /// Deterministic seed of this cave.
    pub seed: u64,
}

/// One elliptical cross-section station along a cave tunnel or chamber spine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpineNode {
    /// Center position in the cave's tangent frame (Y is metres above sea level).
    pub pos: Vec3,
    /// Horizontal half-width in metres.
    pub rx: f32,
    /// Vertical half-height in metres.
    pub ry: f32,
}

/// Side alcove branching off the main cavern chamber.
#[derive(Debug, Clone, PartialEq)]
pub struct CaveAlcove {
    /// Spine nodes from chamber wall to alcove tip.
    pub spine: Vec<SpineNode>,
}

/// Full deterministic layout of one cave: portal, main tunnel/chamber spine, and side alcoves.
#[derive(Debug, Clone, PartialEq)]
pub struct CavePlan {
    /// Stable cave identifier.
    pub id: CaveId,
    /// Deterministic seed of this cave.
    pub seed: u64,
    /// Cave biome theme.
    pub biome: CaveBiome,
    /// Entrance center in caller's chart coordinates.
    pub entry_xz: Vec2,
    /// Entrance ground height in metres above sea level.
    pub entry_y: f32,
    /// Entrance portal radius in metres.
    pub entry_radius: f32,
    /// Main descending tunnel and grand cavern chamber spine.
    pub main_spine: Vec<SpineNode>,
    /// One or two side crystal alcoves branching off the main chamber.
    pub alcoves: Vec<CaveAlcove>,
    /// Center of the main cavern chamber in cave frame coordinates.
    pub chamber_center: Vec3,
    /// Walkable floor Y of the main cavern chamber.
    pub chamber_floor_y: f32,
    /// Ceiling Y of the main cavern chamber.
    pub chamber_ceiling_y: f32,
    /// Horizontal radius of the main cavern chamber.
    pub chamber_radius: f32,
}

/// Returns the cave portal in 320 m spatial `cell` of `terrain`, or nothing when the cell has no cave.
pub fn portal_in_cell(terrain: &Terrain, cell: IVec2) -> Option<CavePortal> {
    let globe = terrain.globe();
    let home = globe.has_ground().then(|| terrain.home(cell, CAVE_CELL))?;
    let face = match terrain.chart() {
        Chart::Face { face, .. } => face as u8 + 1,
        _ => 0,
    };
    let key =
        (u64::from(face) << 56) ^ ((home.id.x as u32 as u64) << 28) ^ (home.id.y as u32 as u64);
    let seed = child(terrain.seed() ^ CAVE_SALT, key);
    let mut rng = Rng::new(seed);
    let (cx, cz) = (rng.below(5) as i32, rng.below(5) as i32);
    if rng.unit() > SPAWN_CHANCE || (face == 0 && home.id == IVec2::ZERO && cx == 0 && cz == 0) {
        return None;
    }
    let entry_radius = 3.6 + rng.unit() as f32 * 1.6;
    let off = Vec2::new(
        cx as f32 * CHUNK_SIZE + 20.0 + rng.unit() as f32 * 24.0,
        cz as f32 * CHUNK_SIZE + 20.0 + rng.unit() as f32 * 24.0,
    );
    let local = home.cell.as_vec2() * CAVE_CELL + off;
    let entry_y = home.view.height(local.x, local.y);
    let pos = home.view.embed(local.x, entry_y, local.y);
    let ok = entry_y > SEA_LEVEL + MIN_SURFACE_Y
        && !home.view.pond_at(local.x, local.y)
        && !home.view.river_at(local.x, local.y)
        && !globe.on_pad(pos, 24.0)
        && !globe.reserve().is_some_and(|r| r.covers(pos, 32.0));
    let biome = match (globe.record().class, rng.below(3)) {
        (PlanetClass::Ice | PlanetClass::Barren | PlanetClass::Lava, 0 | 1)
        | (PlanetClass::Desert, 1) => CaveBiome::CrystalSpire,
        (PlanetClass::Island | PlanetClass::Ocean, 0) | (_, 0) => CaveBiome::BioluminescentGrotto,
        (_, 1) => CaveBiome::CrystalSpire,
        _ => CaveBiome::MycelialDepths,
    };
    ok.then_some(CavePortal {
        id: CaveId {
            face,
            cell_x: home.id.x,
            cell_z: home.id.y,
        },
        entry_xz: local + home.offset,
        entry_y,
        entry_radius,
        biome,
        seed,
    })
}

/// Fast O(1) lookup returning the cave portal containing `chart_xz`, if any.
pub fn cave_portal_at(terrain: &Terrain, chart_xz: Vec2) -> Option<CavePortal> {
    let cell = (chart_xz / CAVE_CELL).floor().as_ivec2();
    let portal = portal_in_cell(terrain, cell)?;
    (portal.entry_xz.distance(chart_xz) <= portal.entry_radius).then_some(portal)
}

/// Fast O(1) lookup returning the cave portal lying inside 64 m chunk `coord`, if any.
pub fn portal_in_chunk(terrain: &Terrain, coord: IVec2) -> Option<CavePortal> {
    let min = coord.as_vec2() * CHUNK_SIZE;
    let cell = ((min + Vec2::splat(CHUNK_SIZE * 0.5)) / CAVE_CELL)
        .floor()
        .as_ivec2();
    let portal = portal_in_cell(terrain, cell)?;
    let inside = portal.entry_xz.x >= min.x
        && portal.entry_xz.x < min.x + CHUNK_SIZE
        && portal.entry_xz.y >= min.y
        && portal.entry_xz.y < min.y + CHUNK_SIZE;
    inside.then_some(portal)
}

/// Picks the initial tunnel yaw among 8 seeded directions that maximizes minimum overburden.
fn best_yaw(view: &Terrain, entry: Vec2, seed: u64) -> f32 {
    let base = Rng::new(child(seed, 0xD12)).unit() as f32 * std::f32::consts::TAU;
    let mut best = (f32::NEG_INFINITY, base);
    for k in 0..8 {
        let yaw = base + k as f32 * std::f32::consts::FRAC_PI_4;
        let dir = Vec2::new(yaw.cos(), yaw.sin());
        let min_h = [16.0_f32, 34.0, 54.0, 76.0]
            .into_iter()
            .map(|d| view.height(entry.x + dir.x * d, entry.y + dir.y * d))
            .fold(f32::INFINITY, f32::min);
        if min_h > best.0 {
            best = (min_h, yaw);
        }
    }
    best.1
}

/// Advances one walkable spine node with slope clamped under 25 degrees and ceiling under surface.
fn step_node(
    view: &Terrain,
    prev: SpineNode,
    entry: Vec2,
    (yaw, horiz, rx, ry): (f32, f32, f32, f32),
    (drop_tan, collar): (f32, bool),
) -> SpineNode {
    let (nx, nz) = (
        prev.pos.x + yaw.cos() * horiz,
        prev.pos.z + yaw.sin() * horiz,
    );
    let max_drop = horiz * MAX_SLOPE_TAN;
    let surf = view.height(entry.x + nx, entry.y + nz);
    let desired = prev.pos.y - horiz * drop_tan;
    let limit = if collar {
        desired
    } else {
        surf - CEILING_CLEARANCE - ry
    };
    let ny = desired
        .min(limit)
        .clamp(prev.pos.y - max_drop, prev.pos.y + max_drop * 0.25);
    let safe_ry = if collar {
        ry
    } else {
        ry.min((surf - CEILING_CLEARANCE - ny).max(0.8))
    };
    SpineNode {
        pos: Vec3::new(nx, ny, nz),
        rx,
        ry: safe_ry,
    }
}

/// Builds one side crystal alcove branching off `start` at direction `yaw`.
fn build_alcove(view: &Terrain, start: SpineNode, entry: Vec2, yaw: f32, seed: u64) -> CaveAlcove {
    let mut rng = Rng::new(seed);
    let mut spine = vec![SpineNode {
        rx: 3.0,
        ry: 2.8,
        ..start
    }];
    let mut heading = yaw;
    for k in 1..=4 {
        heading += (rng.unit() as f32 - 0.5) * 0.35;
        let rx = if k == 4 {
            1.4
        } else {
            2.8 + rng.unit() as f32 * 1.1
        };
        let ry = if k == 4 {
            1.3
        } else {
            2.5 + rng.unit() as f32 * 0.9
        };
        spine.push(step_node(
            view,
            spine[k - 1],
            entry,
            (heading, 4.2, rx, ry),
            (0.06, false),
        ));
    }
    CaveAlcove { spine }
}

/// Builds the full walkable tunnel, grand cavern chamber, and alcoves for `portal`.
pub fn build_plan(terrain: &Terrain, portal: CavePortal) -> CavePlan {
    let mut rng = Rng::new(child(portal.seed, 0x51A));
    let mut yaw = best_yaw(terrain, portal.entry_xz, portal.seed);
    let mut spine = vec![SpineNode {
        pos: Vec3::new(0.0, portal.entry_y - 2.2, 0.0),
        rx: portal.entry_radius,
        ry: portal.entry_radius * 0.85,
    }];
    for k in 1..=10 {
        yaw += (k as f32 * 0.7 + rng.unit() as f32).sin() * 0.22;
        let rx = portal.entry_radius * (0.86 + rng.unit() as f32 * 0.24);
        let ry = (rx * 0.82).clamp(2.8, 4.4);
        let slope = if k <= 3 { 0.42 } else { 0.34 };
        let next = step_node(
            terrain,
            *spine.last().unwrap(),
            portal.entry_xz,
            (yaw, 4.8, rx, ry),
            (slope, k == 1),
        );
        let surf = terrain.height(
            portal.entry_xz.x + next.pos.x,
            portal.entry_xz.y + next.pos.z,
        );
        if k >= 2 && next.pos.y + next.ry >= surf - CEILING_CLEARANCE {
            break;
        }
        spine.push(next);
    }
    let (ch_rx, ch_ry) = (6.2 + rng.unit() as f32 * 4.4, 3.6 + rng.unit() as f32 * 2.2);
    for k in 1..=6 {
        let swell = (k as f32 / 6.0 * std::f32::consts::PI).sin().max(0.28);
        let prev = *spine.last().unwrap();
        let next = step_node(
            terrain,
            prev,
            portal.entry_xz,
            (yaw, 5.2, (ch_rx * swell).max(3.2), (ch_ry * swell).max(2.6)),
            (0.08, false),
        );
        let surf = terrain.height(
            portal.entry_xz.x + next.pos.x,
            portal.entry_xz.y + next.pos.z,
        );
        if next.pos.y + next.ry >= surf - CEILING_CLEARANCE {
            break;
        }
        spine.push(next);
    }
    let mid = spine[spine.len().saturating_sub(3).min(spine.len() - 1)];
    let alcoves = (0..1 + rng.below(2) as usize)
        .map(|a| {
            build_alcove(
                terrain,
                mid,
                portal.entry_xz,
                yaw + if a == 0 { 1.25 } else { -1.25 },
                child(portal.seed, 0xA10 + a as u64),
            )
        })
        .collect();
    CavePlan {
        id: portal.id,
        seed: portal.seed,
        biome: portal.biome,
        entry_xz: portal.entry_xz,
        entry_y: portal.entry_y,
        entry_radius: portal.entry_radius,
        main_spine: spine,
        alcoves,
        chamber_center: mid.pos,
        chamber_floor_y: mid.pos.y - mid.ry * 0.42,
        chamber_ceiling_y: mid.pos.y + mid.ry * 0.92,
        chamber_radius: ch_rx * 0.85,
    }
}

/// Returns all full cave plans whose surface portal lies within `radius` metres of `chart_xz`.
pub fn caves_near(terrain: &Terrain, chart_xz: Vec2, radius: f32) -> Vec<CavePlan> {
    let min = ((chart_xz - Vec2::splat(radius)) / CAVE_CELL)
        .floor()
        .as_ivec2();
    let max = ((chart_xz + Vec2::splat(radius)) / CAVE_CELL)
        .floor()
        .as_ivec2();
    let mut out = Vec::new();
    for cx in min.x..=max.x {
        for cz in min.y..=max.y {
            if let Some(portal) = portal_in_cell(terrain, IVec2::new(cx, cz))
                && portal.entry_xz.distance(chart_xz) <= radius
            {
                out.push(build_plan(terrain, portal));
            }
        }
    }
    out
}
