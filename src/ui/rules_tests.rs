//! Rule tests: nothing in the interface code is rounded, and the fades ease out and stay in range.

use std::path::{Path, PathBuf};

use super::motion;

/// Folders whose code builds interface nodes.
const FOLDERS: [&str; 4] = ["ui", "map", "computer", "menus"];

/// Files allowed to round a corner: the rings of orbits on the map picture, until the owner
/// says whether the picture is exempt.
const EXEMPT: [&str; 1] = ["computer/orbits.rs"];

/// Returns every Rust file under `dir`.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// No node of the interface sets a border radius other than zero.
#[test]
fn no_corner_is_rounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    for folder in FOLDERS {
        sources(&root.join(folder), &mut files);
    }
    assert!(files.len() > 50, "the scan found {} files", files.len());
    for file in files {
        let name = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if EXEMPT.contains(&name.as_str()) || name.contains("rules_tests") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for (at, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or_default();
            let rounded = code.contains("BorderRadius") && !code.contains("ZERO");
            let rounded = rounded && !code.trim_start().starts_with("use ");
            assert!(!rounded, "{name}:{}: {line}", at + 1);
        }
    }
}

/// A fade eases out, never overshoots, and takes [`motion::FADE`] seconds for the whole way.
#[test]
fn fades_ease_out_and_stay_in_range() {
    assert_eq!(motion::ease_out(0.0), 0.0);
    assert_eq!(motion::ease_out(1.0), 1.0);
    assert!(motion::ease_out(0.5) > 0.5);
    assert_eq!(motion::ease_out(2.0), 1.0);
    assert_eq!(motion::approach(0.0, true, motion::FADE), 1.0);
    assert_eq!(motion::approach(1.0, false, 10.0), 0.0);
    assert!((motion::approach(0.0, true, motion::FADE * 0.5) - 0.5).abs() < 1e-6);
}
