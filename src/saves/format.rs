//! Human readable times for the world list.

/// Describes how long ago `then` was, relative to `now`, both in unix seconds.
pub fn ago(now: u64, then: u64) -> String {
    let seconds = now.saturating_sub(then);
    let (minutes, hours, days) = (seconds / 60, seconds / 3600, seconds / 86_400);
    match (days, hours, minutes) {
        (0, 0, 0) => "just now".to_string(),
        (0, 0, m) => format!("{m} min ago"),
        (0, h, _) => format!("{h} h ago"),
        (1, _, _) => "1 day ago".to_string(),
        (d, _, _) => format!("{d} days ago"),
    }
}

/// Formats a play time as `1 h 24 min`, or `12 min` under an hour.
pub fn play_time(seconds: f64) -> String {
    let minutes = (seconds.max(0.0) / 60.0) as u64;
    if minutes < 60 {
        format!("{minutes} min")
    } else {
        format!("{} h {} min", minutes / 60, minutes % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Relative times pick the largest fitting unit.
    #[test]
    fn describes_elapsed_time() {
        assert_eq!(ago(1000, 990), "just now");
        assert_eq!(ago(1000, 1000 - 300), "5 min ago");
        assert_eq!(ago(100_000, 100_000 - 3 * 3600 - 5), "3 h ago");
        assert_eq!(ago(1_000_000, 1_000_000 - 2 * 86_400), "2 days ago");
        assert_eq!(ago(10, 20), "just now");
    }

    /// Play time shows hours only once it passes one.
    #[test]
    fn formats_play_time() {
        assert_eq!(play_time(5040.0), "1 h 24 min");
        assert_eq!(play_time(725.0), "12 min");
        assert_eq!(play_time(-3.0), "0 min");
    }
}
