//! Player names: what a name typed by anyone is turned into before others see it.

use super::NAME_MAX;

/// Returns `raw` without control characters, without space at either end and cut to
/// [`NAME_MAX`] characters. The result is empty when nothing is left.
pub fn clean_name(raw: &str) -> String {
    let kept: String = raw.chars().filter(|c| !c.is_control()).collect();
    let cut: String = kept.trim().chars().take(NAME_MAX).collect();
    cut.trim_end().to_string()
}
