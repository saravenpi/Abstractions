//! Turning a single-player world of the game into a server's world.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

use super::World;
use crate::chest::{Chests, valid_dir, valid_prop};
use crate::ground::Pile;
use crate::ids::BodyId;
use crate::log::Changes;
use crate::wire::{DIGS_MAX, DigOp, TREE_ID_MAX, TREES_MAX, TreeFell};

/// Minimum chop count in single-player saves representing a completely felled tree.
const TREE_CHOPS: u8 = 3;

/// Place body subset read from a single-player world file so felled trees keep their body.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GamePlace {
    /// Body the single-player character last stood on.
    body: BodyId,
}

/// Terrain crater stamp shape read from a single-player world file (`DigStamp`).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ImportDig {
    /// Planet where the crater was dug (accepts `planet` or `body` in TOML).
    #[serde(alias = "body")]
    planet: BodyId,
    /// Unit surface direction from the planet centre.
    dir: [f64; 3],
    /// Crater radius along the surface, in metres.
    radius: f32,
    /// Peak vertical depression at the crater centre, in metres.
    depth: f32,
}

/// The keys of the game's world file a server needs. Every other key is ignored, and a
/// missing one takes its default, as in the game.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GameWorld {
    /// Display name of the world.
    name: Option<String>,
    /// Seed of the universe.
    seed: u64,
    /// The home rule of the world, 0 for a file made before it had one.
    home: u32,
    /// Universe time in seconds.
    clock_seconds: f64,
    /// Where the player last stood, used as the body for imported felled trees.
    place: Option<GamePlace>,
    /// What the player discovered, kept as a table and never interpreted.
    atlas: Option<toml::Table>,
    /// What the player changed, oldest first.
    changes: Changes,
    /// The things on the ground of the world.
    ground: Pile,
    /// Choppable tree damage and felled tree ids (`"<id>:<hits>"` or raw `<id>`).
    felled: Vec<String>,
    /// Player-dug terrain crater stamps.
    digs: Vec<ImportDig>,
    /// Placed chests and torches, if present in the save.
    chests: Chests,
}

/// Extracts the tree id from a saved row (`"<id>:<hits>"` or raw `<id>`), skipping incomplete
/// chops under [`TREE_CHOPS`].
fn tree_id(row: &str) -> Option<&str> {
    match row.split_once(':') {
        Some((id, rest)) => match rest.parse::<u8>() {
            Ok(hits) if hits < TREE_CHOPS => None,
            Ok(_) => Some(id),
            Err(_) => Some(row),
        },
        None => Some(row),
    }
}

/// Converts single-player `felled` rows into deduplicated [`TreeFell`] records on `body`.
fn import_trees(felled: Vec<String>, body: BodyId) -> Vec<TreeFell> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in felled {
        let Some(id) = tree_id(&row) else {
            continue;
        };
        if !id.is_empty() && id.len() <= TREE_ID_MAX && seen.insert(id.to_string()) {
            out.push(TreeFell {
                body,
                id: id.to_string(),
            });
            if out.len() >= TREES_MAX {
                break;
            }
        }
    }
    out
}

/// Converts single-player terrain craters into bounded [`DigOp`] entries.
fn import_digs(digs: Vec<ImportDig>) -> Vec<DigOp> {
    digs.into_iter()
        .map(|d| DigOp {
            body: d.planet,
            dir: d.dir.map(|x| x as f32),
            radius: d.radius,
            depth: d.depth,
        })
        .filter(|op| {
            valid_dir(op.dir)
                && op.radius.is_finite()
                && op.radius > 0.0
                && op.depth.is_finite()
                && op.depth > 0.0
        })
        .take(DIGS_MAX)
        .collect()
}

/// Normalizes imported placed props, retaining valid props and advancing `next_id`.
fn import_chests(raw: Chests) -> Chests {
    let props: Vec<_> = raw.props.into_iter().filter(valid_prop).collect();
    let mut chests = Chests::new(props);
    chests.next_id = chests.next_id.max(raw.next_id);
    chests
}

/// Returns the server world held in the game's world file at `path`, with the things on its
/// ground, felled trees, digs, and chests. The atlas comes back as TOML text, or empty when the
/// file has none, so the first player builds a new one.
pub fn import(path: &Path) -> io::Result<World> {
    let text = fs::read_to_string(path)?;
    let game: GameWorld =
        toml::from_str(&text).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let atlas = match game.atlas {
        Some(table) => toml::to_string(&table).map_err(io::Error::other)?,
        None => String::new(),
    };
    let save_body = game.place.map_or_else(BodyId::default, |p| p.body);
    Ok(World {
        name: game.name.unwrap_or_else(|| World::default().name),
        seed: game.seed,
        home: game.home,
        clock: game.clock_seconds,
        changes: game.changes,
        atlas,
        ground: game.ground.lying,
        chests: import_chests(game.chests),
        trees: import_trees(game.felled, save_body),
        digs: import_digs(game.digs),
        ..World::default()
    })
}
