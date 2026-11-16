//! World ids: file-safe slugs made unique among existing ids.

/// Turns a name into ASCII lowercase words joined by dashes. Never returns an empty string.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "world".to_string()
    } else {
        out
    }
}

/// Returns `base`, or `base` with the first free numeric suffix starting at 2.
pub fn unique_id(base: &str, taken: &[String]) -> String {
    if !taken.iter().any(|id| id == base) {
        return base.to_string();
    }
    (2u32..)
        .map(|n| format!("{base}-{n}"))
        .find(|id| !taken.contains(id))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Slugs are lowercase ASCII with single dashes and never empty.
    #[test]
    fn slugs_are_clean() {
        assert_eq!(slug("  My Big -- World! "), "my-big-world");
        assert_eq!(slug("Été"), "t");
        assert_eq!(slug("???"), "world");
        assert_eq!(slug(""), "world");
    }

    /// A taken id gets the first free numeric suffix.
    #[test]
    fn ids_become_unique() {
        let taken = vec!["isle".to_string(), "isle-2".to_string()];
        assert_eq!(unique_id("isle", &taken), "isle-3");
        assert_eq!(unique_id("reef", &taken), "reef");
    }
}
