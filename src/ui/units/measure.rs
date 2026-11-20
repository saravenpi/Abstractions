//! Speeds and times as plain words.

use super::number::plain;
use crate::universe::SPEED_OF_LIGHT;

/// Returns a speed: "12 m/s" on foot, "7.5 km/s" in space and "1 200 x light speed" in warp.
pub fn speed(metres_per_second: f64) -> String {
    let speed = metres_per_second.abs();
    match speed {
        s if s < 999.5 => format!("{} m/s", plain(s)),
        s if s < SPEED_OF_LIGHT => format!("{} km/s", plain(s / 1e3)),
        s => format!("{} x light speed", plain(s / SPEED_OF_LIGHT)),
    }
}

/// Returns a time: "45 s", "3 min", "2 h 10 min", "5 days" or "3 years".
pub fn duration(seconds: f64) -> String {
    let seconds = seconds.max(0.0).round() as u64;
    let (minutes, hours, days) = (seconds / 60, seconds / 3600, seconds / 86_400);
    match (days, hours, minutes) {
        (0, 0, 0) => format!("{seconds} s"),
        (0, 0, m) => format!("{m} min"),
        (0, h, _) if minutes % 60 == 0 => format!("{h} h"),
        (0, h, _) => format!("{h} h {} min", minutes % 60),
        (1, _, _) => "1 day".to_string(),
        (d, _, _) if d < 730 => format!("{d} days"),
        (d, _, _) => format!("{} years", d / 365),
    }
}
