//! Pure helpers for the typed world name.

/// Maximum number of characters a world name may have.
pub const MAX_CHARS: usize = 24;

/// Appends the printable characters of `text` and removes one when `backspace` is set.
pub fn edit(name: &mut String, backspace: bool, text: &str) {
    edit_to(name, backspace, text, MAX_CHARS);
}

/// Edits `value` as [`edit`] does, keeping it to at most `cap` characters.
pub fn edit_to(value: &mut String, backspace: bool, text: &str, cap: usize) {
    if backspace {
        value.pop();
    }
    for c in text.chars().filter(|c| !c.is_control()) {
        if value.chars().count() < cap {
            value.push(c);
        }
    }
}

/// Returns `value` followed by the caret of a text field that is being typed in.
pub fn with_caret(value: &str) -> String {
    format!("{value}_")
}

/// Why a typed world name is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Nothing but spaces was typed.
    Empty,
    /// The name has more than [`MAX_CHARS`] characters.
    TooLong,
    /// Another world already has this name.
    Taken,
}

impl Refusal {
    /// Returns the short message shown to the player.
    pub fn message(self) -> &'static str {
        match self {
            Refusal::Empty => "A world needs a name",
            Refusal::TooLong => "A name has at most 24 characters",
            Refusal::Taken => "Another world has this name",
        }
    }
}

/// Returns the name to store for a typed one: trimmed, not empty, not too long, and different
/// from every name in `others` whatever the letter case.
pub fn validate<'a>(
    typed: &str,
    others: impl IntoIterator<Item = &'a str>,
) -> Result<String, Refusal> {
    let name = typed.trim();
    if name.is_empty() {
        return Err(Refusal::Empty);
    }
    if name.chars().count() > MAX_CHARS {
        return Err(Refusal::TooLong);
    }
    let lower = name.to_lowercase();
    match others
        .into_iter()
        .any(|other| other.to_lowercase() == lower)
    {
        true => Err(Refusal::Taken),
        false => Ok(name.to_string()),
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

    /// A name is trimmed, and refused when empty, too long or already used by another world.
    #[test]
    fn validates_typed_names() {
        let others = ["Isle", "Reef one"];
        assert_eq!(validate("  Atoll  ", others), Ok("Atoll".to_string()));
        assert_eq!(validate("", others), Err(Refusal::Empty));
        assert_eq!(validate("   \t ", others), Err(Refusal::Empty));
        assert_eq!(validate("isle", others), Err(Refusal::Taken));
        assert_eq!(validate(" REEF ONE ", others), Err(Refusal::Taken));
        assert_eq!(
            validate(&"x".repeat(MAX_CHARS), others).map(|n| n.len()),
            Ok(MAX_CHARS)
        );
        assert_eq!(
            validate(&"x".repeat(MAX_CHARS + 1), others),
            Err(Refusal::TooLong)
        );
        let padded = format!("  {}  ", "y".repeat(MAX_CHARS));
        assert_eq!(validate(&padded, others).map(|n| n.len()), Ok(MAX_CHARS));
        assert_eq!(
            Refusal::TooLong.message(),
            format!("A name has at most {MAX_CHARS} characters")
        );
        assert_eq!(with_caret("Isle"), "Isle_");
    }
}
