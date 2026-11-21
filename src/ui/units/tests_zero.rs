//! Tests of the rule that no quantity is ever written as a negative zero: the one rounding
//! function drops the sign of a value that rounds to zero, every unit goes through it, and no
//! other file of the interface rounds a number by hand.

use std::fs;
use std::path::Path;

use super::tests_rules::{UI_PATHS, walk};
use super::{distance, duration, fixed, gravity_si, mass, plain, pressure_bar, speed};

/// A negative zero and a small negative number that rounds to zero are written without a sign,
/// with any number of decimals. A number that keeps a digit keeps its sign.
#[test]
fn a_zero_is_never_negative() {
    assert_eq!(fixed(-0.0_f64, 1), "0.0");
    assert_eq!(fixed(-0.0_f32, 2), "0.00");
    assert_eq!(fixed(-0.04_f32, 1), "0.0");
    assert_eq!(fixed(-0.4, 0), "0");
    assert_eq!(fixed(0.0, 0), "0");
    assert_eq!(fixed(-0.06, 1), "-0.1");
    assert_eq!(fixed(-2.3, 1), "-2.3");
    assert_eq!(fixed(1.5, 1), "1.5");
}

/// The sum of no masses is a negative zero in Rust, which is what an empty pack weighs: it is
/// written as zero.
#[test]
fn the_sum_of_nothing_reads_as_zero() {
    let nothing: f32 = std::iter::empty::<f32>().sum();
    assert_eq!(fixed(nothing, 1), "0.0");
    assert_eq!(mass(f64::from(nothing)), "0 g");
}

/// Every unit writes a negative zero as its plain zero.
#[test]
fn every_unit_writes_zero_without_a_sign() {
    let written = [
        plain(-0.0),
        mass(-0.0),
        distance(-0.0),
        speed(-0.0),
        duration(-0.0),
        gravity_si(-0.0),
        pressure_bar(-0.0),
    ];
    for words in written {
        assert!(!words.contains('-'), "{words}");
    }
}

/// Returns true when `line` rounds a number in a format string: a precision such as `{:.1}`,
/// `{v:.2}` or `{value:.digits$}`.
fn rounds(line: &str) -> bool {
    line.match_indices(":.").any(|(at, _)| {
        let before = line[..at].rfind('{');
        let closed = line[..at].rfind('}');
        let open = before.is_some_and(|open| closed.is_none_or(|closed| closed < open));
        let digit = line[at + 2..].chars().next();
        open && digit.is_some_and(|c| c.is_ascii_alphanumeric())
    })
}

/// The precision check sees the three ways a format string rounds, and nothing else.
#[test]
fn the_check_sees_a_rounding() {
    assert!(rounds("format!(\"{:.1} kg\", load)"));
    assert!(rounds("\"{value:.digits$}{unit}\""));
    assert!(rounds("write!(out, \"{} {lat:.1}\", a)"));
    assert!(!rounds("format!(\"{} kg\", fixed(load, 1))"));
    assert!(!rounds("let ratio: f32 = a.min(b);"));
    assert!(!rounds("crate::ui::units::fixed(value, 1)"));
}

/// No file of the interface rounds a number in a format string: each calls [`fixed`], so none
/// can write a negative zero.
#[test]
fn no_file_rounds_a_number_by_hand() {
    let mut files = Vec::new();
    UI_PATHS
        .iter()
        .for_each(|dir| walk(Path::new(dir), &mut files));
    let mut found = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        let code = source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"));
        for line in code.filter(|line| rounds(line)) {
            found.push(format!("{}: {}", file.to_string_lossy(), line.trim()));
        }
    }
    assert!(found.is_empty(), "{found:#?}");
}
