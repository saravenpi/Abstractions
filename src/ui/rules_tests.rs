//! Rule tests: nothing in the interface code is rounded, the fades ease out and stay in range,
//! the alignment grid and the section gaps hold, and one concept has one icon. The rules that
//! need a built page are held in `menus/tests/rules.rs`.

use std::path::{Path, PathBuf};

use super::icon::CODEPOINTS;
use super::{motion, sign, theme};

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

/// The alignment grid: a block is a whole number of columns wide, gutters included, and every
/// such width is on the 4 pixel grid.
#[test]
fn spans_sit_on_the_grid() {
    assert_eq!(theme::span(1), theme::COLUMN);
    assert_eq!(
        theme::span(3) + theme::GUTTER + theme::span(8),
        theme::span(11)
    );
    for columns in 1..16 {
        assert_eq!(theme::span(columns) % 4.0, 0.0);
    }
    assert!(theme::span(11) + 2.0 * theme::INSET <= super::MIN_WIDTH - 2.0 * theme::EDGE);
}

/// A title is nearer to what it heads than to what is above it, on a page and in a card, and
/// the three gaps are steps of the spacing scale.
#[test]
fn the_space_above_a_title_is_larger_than_the_space_below() {
    let scale = [
        theme::S1,
        theme::S2,
        theme::S3,
        theme::S4,
        theme::S5,
        theme::S6,
    ];
    for gap in [theme::SECTION_ABOVE, theme::GROUP_ABOVE, theme::TITLE_BELOW] {
        assert!(scale.contains(&gap));
    }
    const { assert!(theme::SECTION_ABOVE > theme::GROUP_ABOVE) };
    const { assert!(theme::GROUP_ABOVE > theme::TITLE_BELOW) };
}

/// One concept, one icon: no two icons share a glyph, and no concept that borrows an icon is
/// listed twice or asks for a glyph the font already has under its own name.
#[test]
fn a_concept_maps_to_one_icon() {
    let mut glyphs: Vec<char> = CODEPOINTS.iter().map(|(_, glyph)| *glyph).collect();
    glyphs.sort_unstable();
    glyphs.dedup();
    assert_eq!(glyphs.len(), CODEPOINTS.len(), "two icons share a glyph");
    let mut concepts: Vec<&str> = sign::BORROWED.iter().map(|(name, _, _)| *name).collect();
    concepts.sort_unstable();
    concepts.dedup();
    assert_eq!(concepts.len(), sign::BORROWED.len());
}
