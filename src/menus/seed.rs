//! Pure helpers for the typed world seed.

/// Maximum number of digits a typed seed may have.
pub const MAX_DIGITS: usize = 18;

/// Appends the digits of `text` and removes one character when `backspace` is set.
pub fn edit(seed: &mut String, backspace: bool, text: &str) {
    if backspace {
        seed.pop();
    }
    for digit in text.chars().filter(char::is_ascii_digit) {
        if seed.len() < MAX_DIGITS {
            seed.push(digit);
        }
    }
}

/// Parses typed digits into a seed, or `None` when the text is empty.
pub fn parse(seed: &str) -> Option<u64> {
    seed.parse().ok()
}

/// Returns a random seed that fits `MAX_DIGITS` digits.
pub fn random() -> u64 {
    rand::random::<u64>() % 10u64.pow(MAX_DIGITS as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Digits append, other characters are ignored and backspace removes one.
    #[test]
    fn edits_digits_only() {
        let mut seed = String::new();
        edit(&mut seed, false, "12a3");
        assert_eq!(seed, "123");
        edit(&mut seed, true, "");
        assert_eq!(seed, "12");
    }

    /// Input stops at the digit cap and every capped seed parses.
    #[test]
    fn caps_length_and_parses() {
        let mut seed = String::new();
        edit(&mut seed, false, &"9".repeat(40));
        assert_eq!(seed.len(), MAX_DIGITS);
        assert!(parse(&seed).is_some());
        assert_eq!(parse(""), None);
        assert!(random() < 10u64.pow(MAX_DIGITS as u32));
    }
}
