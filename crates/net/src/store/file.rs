//! Reading and writing `world.toml`: atomic writes and the last three copies.

use std::fs::{self, File};
use std::hash::{BuildHasher, Hasher, RandomState};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::{World, explored};

/// Name of the world file in the data folder.
pub const FILE: &str = "world.toml";

/// Copies of the world file kept beside it, as `world.toml.1` (newest) to `world.toml.3`.
const COPIES: u32 = 3;

/// Shortest time between two rotations of the copies.
const ROTATE_EVERY: Duration = Duration::from_secs(600);

/// Seeds stay under this bound, as the game's random seeds do, so they fit 18 digits.
const SEED_BOUND: u64 = 1_000_000_000_000_000_000;

/// Returns the world of `dir/world.toml` with the explored ground of `dir/explored.bin`. When there is none, creates the folder and a new
/// world named `name_hint` with a random seed, and writes it.
pub fn load_or_create(dir: &Path, name_hint: &str) -> io::Result<World> {
    let path = dir.join(FILE);
    match fs::read_to_string(&path) {
        Ok(text) => {
            let world: World = toml::from_str(&text)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            let explored = explored::load(dir)?;
            Ok(World { explored, ..world })
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let world = World {
                name: name_hint.to_string(),
                seed: random_seed(),
                ..World::default()
            };
            fs::create_dir_all(dir)?;
            save(dir, &world)?;
            Ok(world)
        }
        Err(error) => Err(error),
    }
}

/// Writes `world` to `dir/world.toml` atomically: a temporary file, flushed to disk, then
/// renamed over the old one. The old file becomes copy 1 at most once every ten minutes. The
/// explored ground goes to `dir/explored.bin` the same way.
pub fn save(dir: &Path, world: &World) -> io::Result<()> {
    save_every(dir, world, ROTATE_EVERY)
}

/// Does what [`save`] does, rotating the copies at most once every `every`.
pub(super) fn save_every(dir: &Path, world: &World, every: Duration) -> io::Result<()> {
    let text = toml::to_string(world).map_err(io::Error::other)?;
    let path = dir.join(FILE);
    let tmp = dir.join(format!("{FILE}.tmp"));
    let mut file = File::create(&tmp)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    if path.exists() && rotation_due(dir, every) {
        rotate(dir, &path)?;
    }
    fs::rename(&tmp, &path)?;
    File::open(dir)?.sync_all()?;
    explored::save(dir, &world.explored)
}

/// Returns the path of copy `n` of the world file.
fn copy(dir: &Path, n: u32) -> PathBuf {
    dir.join(format!("{FILE}.{n}"))
}

/// Returns true when copy 1 is missing or older than `every`.
fn rotation_due(dir: &Path, every: Duration) -> bool {
    let modified = fs::metadata(copy(dir, 1)).and_then(|meta| meta.modified());
    modified.map_or(true, |time| {
        SystemTime::now().duration_since(time).unwrap_or_default() >= every
    })
}

/// Shifts every copy one step older, dropping the oldest, and copies `path` as copy 1. The
/// world file itself stays in place, so a crash here loses nothing.
fn rotate(dir: &Path, path: &Path) -> io::Result<()> {
    for n in (1..COPIES).rev() {
        let from = copy(dir, n);
        if from.exists() {
            fs::rename(&from, copy(dir, n + 1))?;
        }
    }
    fs::copy(path, copy(dir, 1)).map(drop)
}

/// Returns a seed from the time and the process, with no random number crate.
fn random_seed() -> u64 {
    let mut hasher = RandomState::new().build_hasher();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    hasher.write_u128(nanos.as_nanos());
    hasher.write_u32(std::process::id());
    hasher.finish() % SEED_BOUND
}
