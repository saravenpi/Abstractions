//! Starting the probe from a world somebody sent, with `PROBE_SAVE=<dir>`, and the tally of the
//! visibility ranges the run hands Bevy, which keeps a GPU slot for each until the app closes.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use abstractions::saves::{WorldEntry, list};
use bevy::camera::visibility::VisibilityRange;

use super::*;

/// Every distinct visibility range the run has handed Bevy.
#[derive(Resource, Default)]
pub struct RangeTally(pub HashSet<VisibilityRange>);

/// Adds the ranges added or changed this frame to the tally.
pub fn tally(
    mut seen: ResMut<RangeTally>,
    ranges: Query<&VisibilityRange, Changed<VisibilityRange>>,
) {
    for range in &ranges {
        seen.0.insert(range.clone());
    }
}

/// Starts the most recently played world of `PROBE_SAVE`: its world files are copied into
/// `SMOKE_DIR/probe-save`, which becomes the save folder, so the autosave never touches the
/// files that were sent. A `settings.toml` beside them is applied to the run.
pub fn resume(world: &mut World) -> Act {
    let from = PathBuf::from(std::env::var("PROBE_SAVE").unwrap_or_default());
    let entry = match adopt(world, &from) {
        Ok(entry) => entry,
        Err(why) => return Act::Fail(why),
    };
    if let Some(older) = entry.save.made_with() {
        return Act::Fail(format!(
            "the world was made by {older} and does not open here"
        ));
    }
    match world.run_system_once(move |mut starter: Starter| starter.start(entry.clone())) {
        Ok(()) => Act::Next,
        Err(e) => Act::Fail(format!("{e:?}")),
    }
}

/// Copies the worlds of `from`, or of its `worlds` folder, into the probe's save folder, applies
/// the settings beside them and returns the most recently played world, described in the log.
fn adopt(world: &mut World, from: &Path) -> Result<WorldEntry, String> {
    let worlds = [from.join("worlds"), from.to_path_buf()];
    let source = worlds
        .iter()
        .find(|dir| dir.is_dir())
        .ok_or_else(|| format!("{} is not a folder", from.display()))?;
    let base = std::env::var("SMOKE_DIR").map_or_else(|_| std::env::temp_dir(), PathBuf::from);
    let dir = base.join("probe-save");
    copy_worlds(source, &dir)?;
    settings_from(world, &from.join("settings.toml"));
    world.insert_resource(SaveDir(dir.clone()));
    let entry = list(&dir).into_iter().next();
    let entry = entry.ok_or_else(|| format!("no readable world file in {}", source.display()))?;
    describe(&entry);
    Ok(entry)
}

/// Copies every file of `from` but the settings into `to`, and logs the files whose text holds a
/// NaN or an infinite number, which TOML writes as `nan` and `inf`.
fn copy_worlds(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    let files = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
    for path in files
        .flatten()
        .map(|file| file.path())
        .filter(|path| path.is_file())
    {
        let name = path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        if name == "settings.toml" {
            continue;
        }
        std::fs::copy(&path, to.join(&name)).map_err(|e| e.to_string())?;
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let broken = text
            .lines()
            .filter(|l| {
                l.split(|c: char| !c.is_alphanumeric())
                    .any(|w| w == "nan" || w == "inf")
            })
            .collect::<Vec<_>>();
        eprintln!(
            "PROBE save file {}: {} lines with nan or inf {broken:?}",
            path.display(),
            broken.len()
        );
    }
    Ok(())
}

/// Applies the settings file at `path` to this run, when there is one and it reads.
fn settings_from(world: &mut World, path: &Path) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    match toml::from_str::<Settings>(&text) {
        Ok(settings) => {
            **world.resource_mut::<Persistent<Settings>>() = settings;
            eprintln!("PROBE applied the settings of {}", path.display());
        }
        Err(e) => eprintln!("PROBE the settings of {} do not read: {e}", path.display()),
    }
}

/// Logs what can be unusual in a world: its clock, its place, its ship and its versions.
fn describe(entry: &WorldEntry) {
    let save = &entry.save;
    eprintln!(
        "PROBE save {}: version {} game {}, seed {}, clock {} s ({:.1} days), played {:.0} s, day phase {}, \
         yaw {} pitch {}, place {:?}, health {:?}, explored bodies {}, changes {}",
        entry.id,
        save.version,
        save.game_version,
        save.seed,
        save.clock_seconds,
        save.clock_seconds / 86_400.0,
        save.play_seconds,
        save.day_phase,
        save.yaw,
        save.pitch,
        save.place,
        save.health,
        save.explored.len(),
        save.changes.entries.len(),
    );
    eprintln!("PROBE save ship {:?}", save.ship);
}
