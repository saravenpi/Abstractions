//! Pure helpers for the typed world name.

/// Maximum number of characters a world name may have.
pub const MAX_CHARS: usize = 24;

/// Appends the printable characters of `text` and removes one when `backspace` is set.
pub fn edit(name: &mut String, backspace: bool, text: &str) {
    if backspace {
        name.pop();
    }
    for c in text.chars().filter(|c| !c.is_control()) {
        if name.chars().count() < MAX_CHARS {
            name.push(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Printable text appends, control characters are dropped and the cap holds.
    #[test]
    fn edits_and_caps() {
        let mut name = String::from("Isle");
        edit(&mut name, false, " one\t\r");
        assert_eq!(name, "Isle one");
        edit(&mut name, true, "");
        assert_eq!(name, "Isle on");
        edit(&mut name, false, &"x".repeat(40));
        assert_eq!(name.chars().count(), MAX_CHARS);
    }
}
