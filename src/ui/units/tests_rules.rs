//! The rule that the interface writes no unit by hand: outside this module, no line of the
//! interface code that calls `format!` writes kilometres, metres, light-years or AU.

use std::fs;
use std::path::Path;

/// The folders of the interface code.
const UI_PATHS: [&str; 6] = [
    "src/computer",
    "src/map",
    "src/menus",
    "src/fauna/guide",
    "src/craft/ui",
    "src/ui",
];

/// What a `format!` line may not write.
const UNITS: [&str; 8] = [" km", " m\"", " m/s", " ly", " AU", "GLY", " gly", " Mm"];

/// Lines allowed to name a unit: the size of a creature, and masses of craft pieces. Each is
/// a file and the text of the line.
const ALLOWED: [(&str, &str); 4] = [
    ("src/fauna/guide/card.rs", "m long"),
    ("src/craft/ui/card.rs", " kg"),
    ("src/craft/ui/paint.rs", " kg"),
    ("src/craft/ui/card.rs", " m/s"),
];

/// Pushes every Rust file under `dir` that is not a test file or this module into `found`.
fn walk(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = path.to_string_lossy().to_string();
        if path.is_dir() && !name.ends_with("ui/units") {
            walk(&path, found);
        } else if name.ends_with(".rs") && !name.contains("test") && !name.contains("rig") {
            found.push(path);
        }
    }
}

/// No `format!` of the interface code writes a unit outside the allowed lines.
#[test]
fn no_file_writes_a_unit_by_hand() {
    let mut files = Vec::new();
    UI_PATHS
        .iter()
        .for_each(|dir| walk(Path::new(dir), &mut files));
    let mut found = Vec::new();
    for file in files {
        let name = file.to_string_lossy().to_string();
        let source = fs::read_to_string(&file).unwrap();
        for line in source.lines().filter(|line| line.contains("format!")) {
            let allowed = ALLOWED
                .iter()
                .any(|(path, text)| name == *path && line.contains(text));
            if !allowed && UNITS.iter().any(|unit| line.contains(unit)) {
                found.push(format!("{name}: {}", line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "{found:#?}");
}
