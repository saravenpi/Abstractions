//! Tests of the name cleaner: what is removed, what is kept and where a long name is cut.

use super::{NAME_MAX, clean_name};

/// Space at the ends and control characters go; inner spaces and other alphabets stay.
#[test]
fn a_name_is_trimmed_and_stripped() {
    assert_eq!(clean_name("  Yann \n"), "Yann");
    assert_eq!(clean_name("a\tb\u{0}c\u{7f}\u{1b}[31m"), "abc[31m");
    assert_eq!(clean_name("Zoé le robot"), "Zoé le robot");
    assert_eq!(clean_name("名前"), "名前");
}

/// A long name is cut to the limit in characters, not bytes, and never ends with a space.
#[test]
fn a_long_name_is_cut() {
    assert_eq!(clean_name(&"é".repeat(40)), "é".repeat(NAME_MAX));
    let spaced = format!("{} tail", "x".repeat(NAME_MAX - 1));
    assert_eq!(clean_name(&spaced), "x".repeat(NAME_MAX - 1));
}

/// A name made of nothing visible is empty.
#[test]
fn an_empty_name_stays_empty() {
    assert_eq!(clean_name(""), "");
    assert_eq!(clean_name(" \r\n\t "), "");
}
